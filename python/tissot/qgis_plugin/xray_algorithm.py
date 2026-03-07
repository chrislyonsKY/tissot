"""QGIS Processing algorithm wrapper for tissot.xray."""

from pathlib import Path
import webbrowser

from qgis.PyQt.QtCore import QVariant
from qgis.core import (
    QgsFeature,
    QgsFeatureSink,
    QgsField,
    QgsFields,
    QgsGeometry,
    QgsProcessing,
    QgsProcessingAlgorithm,
    QgsProcessingException,
    QgsProcessingOutputString,
    QgsProcessingParameterBoolean,
    QgsProcessingParameterFeatureSink,
    QgsProcessingParameterFeatureSource,
    QgsWkbTypes,
)

from . import tissot


class TissotXrayAlgorithm(QgsProcessingAlgorithm):
    """Run Tissot Projection X-Ray against an input vector layer."""

    INPUT = "INPUT"
    OPEN_REPORT = "OPEN_REPORT"
    OUTPUT_FINDINGS = "OUTPUT_FINDINGS"
    OUTPUT_RECOMMENDED_EPSG = "OUTPUT_RECOMMENDED_EPSG"

    def name(self):
        return "xray"

    def displayName(self):
        return self.tr("Projection X-Ray")

    def group(self):
        return self.tr("Tissot")

    def groupId(self):
        return "tissot"

    def shortHelpString(self):
        return self.tr(
            "Runs tissot.xray on an input layer and returns per-feature "
            "distortion findings plus a recommended EPSG code."
        )

    def initAlgorithm(self, config=None):
        self.addParameter(
            QgsProcessingParameterFeatureSource(
                self.INPUT,
                self.tr("Input layer"),
                [QgsProcessing.TypeVectorAnyGeometry],
            )
        )
        self.addParameter(
            QgsProcessingParameterBoolean(
                self.OPEN_REPORT,
                self.tr("Open visual report in browser"),
                defaultValue=True,
            )
        )
        self.addParameter(
            QgsProcessingParameterFeatureSink(
                self.OUTPUT_FINDINGS,
                self.tr("X-Ray findings"),
                type=QgsProcessing.TypeVectorAnyGeometry,
            )
        )

        self.addOutput(
            QgsProcessingOutputString(
                self.OUTPUT_RECOMMENDED_EPSG,
                self.tr("Recommended EPSG"),
            )
        )

    def processAlgorithm(self, parameters, context, feedback):
        source = self.parameterAsSource(parameters, self.INPUT, context)
        open_report = self.parameterAsBool(parameters, self.OPEN_REPORT, context)

        if source is None:
            raise QgsProcessingException(self.tr("Invalid input layer."))

        feedback.pushInfo(self.tr("Preparing input data..."))
        input_path = self.parameterAsCompatibleSourceLayerPath(
            parameters, self.INPUT, context,
            compatibleFormats=['geojson'], preferredFormat='geojson',
            feedback=feedback,
        )
        feedback.setProgress(15)

        try:
            feedback.pushInfo(self.tr("Running tissot.xray..."))
            report = tissot.xray(input_path)
        except Exception as exc:  # pylint: disable=broad-exception-caught
            feedback.reportError(f"tissot.xray failed: {exc}", fatalError=True)
            raise QgsProcessingException(str(exc)) from exc

        feedback.setProgress(55)

        findings = self._safe_get(report, "findings", "distortions", default=[]) or []
        recommended_epsg = self._extract_recommended_epsg(report)
        source_index = self._build_feature_index(source)

        fields = QgsFields()
        fields.append(QgsField("metric", QVariant.String))
        fields.append(QgsField("value", QVariant.Double))
        fields.append(QgsField("severity", QVariant.String))
        fields.append(QgsField("message", QVariant.String))
        fields.append(QgsField("feature_id", QVariant.String))

        sink, dest_id = self.parameterAsSink(
            parameters,
            self.OUTPUT_FINDINGS,
            context,
            fields,
            QgsWkbTypes.Unknown,
            source.sourceCrs(),
        )
        if sink is None:
            raise QgsProcessingException(self.tr("Could not create output sink."))

        total = max(len(findings), 1)
        emitted = 0
        for index, item in enumerate(findings):
            if feedback.isCanceled():
                break
            feature = QgsFeature(fields)
            feature.setAttribute("metric", self._safe_get(item, "metric", default="distortion"))
            feature.setAttribute(
                "value",
                float(self._safe_get(item, "value", "distortion", "metric_value", default=0.0) or 0.0),
            )
            feature.setAttribute("severity", self._safe_get(item, "severity", default="info"))
            feature.setAttribute("message", self._safe_get(item, "message", default=""))
            feature.setAttribute(
                "feature_id", str(self._safe_get(item, "feature_id", "id", default=""))
            )

            geometry = self._geometry_for_finding(item, source_index)
            if geometry is None:
                continue

            feature.setGeometry(geometry)
            sink.addFeature(feature, QgsFeatureSink.FastInsert)
            emitted += 1
            feedback.setProgress(55 + int(((index + 1) / total) * 35))

        feedback.pushInfo(self.tr(f"Exported {emitted} x-ray findings."))

        report_ref = self._safe_get(report, "report_url", "report_path", "html_report")
        if open_report and report_ref:
            self._open_report(report_ref, feedback)

        feedback.setProgress(100)
        return {
            self.OUTPUT_FINDINGS: dest_id,
            self.OUTPUT_RECOMMENDED_EPSG: str(recommended_epsg),
        }

    def createInstance(self):
        return TissotXrayAlgorithm()

    def _safe_get(self, value, *names, default=None):
        for name in names:
            if isinstance(value, dict) and name in value:
                return value[name]
            if hasattr(value, name):
                return getattr(value, name)
        return default

    def _extract_recommended_epsg(self, report):
        recommendations = self._safe_get(report, "recommendations", default=[]) or []
        if recommendations:
            first = recommendations[0]
            epsg = self._safe_get(first, "epsg", "code", default="")
            if epsg:
                return epsg

        nested = self._safe_get(report, "recommended", "recommendation")
        if nested is not None:
            epsg = self._safe_get(nested, "epsg", "code", default="")
            if epsg:
                return epsg
        return ""

    def _build_feature_index(self, source_layer):
        index = {}
        for src_feature in source_layer.getFeatures():
            index[str(src_feature.id())] = src_feature.geometry()
            for attr_name in ("id", "fid", "feature_id"):
                if attr_name in src_feature.fields().names():
                    value = src_feature.attribute(attr_name)
                    if value is not None:
                        index[str(value)] = src_feature.geometry()
        return index

    def _coerce_geometry(self, raw_geometry):
        if raw_geometry is None:
            return None

        if isinstance(raw_geometry, QgsGeometry):
            if raw_geometry.isNull():
                return None
            return raw_geometry

        if isinstance(raw_geometry, dict):
            wkt_value = raw_geometry.get("wkt")
            if wkt_value:
                parsed = QgsGeometry.fromWkt(str(wkt_value))
                return parsed if parsed and not parsed.isNull() else None
            if "x" in raw_geometry and "y" in raw_geometry:
                parsed = QgsGeometry.fromWkt(
                    f"POINT({raw_geometry['x']} {raw_geometry['y']})"
                )
                return parsed if parsed and not parsed.isNull() else None

        if isinstance(raw_geometry, str):
            parsed = QgsGeometry.fromWkt(raw_geometry)
            return parsed if parsed and not parsed.isNull() else None

        if isinstance(raw_geometry, (bytes, bytearray)):
            parsed = QgsGeometry()
            try:
                parsed.fromWkb(bytes(raw_geometry))
                return parsed if not parsed.isNull() else None
            except Exception:  # pylint: disable=broad-exception-caught
                return None

        return None

    def _geometry_for_finding(self, finding, source_index):
        geometry = self._coerce_geometry(self._safe_get(finding, "geometry", "geom"))
        if geometry is not None:
            return geometry

        feature_id = self._safe_get(finding, "feature_id", "id")
        if feature_id is None:
            return None
        return source_index.get(str(feature_id))

    def _open_report(self, report_ref, feedback):
        report_text = str(report_ref)
        try:
            if report_text.startswith(("http://", "https://", "file://")):
                webbrowser.open(report_text)
            else:
                webbrowser.open(Path(report_text).resolve().as_uri())
            feedback.pushInfo(self.tr("Opened visual report in browser."))
        except Exception as exc:  # pylint: disable=broad-exception-caught
            feedback.reportError(f"Could not open visual report: {exc}")
