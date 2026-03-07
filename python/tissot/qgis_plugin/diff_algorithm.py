"""QGIS Processing algorithm wrapper for tissot.diff."""

from pathlib import Path
import webbrowser

from qgis.PyQt.QtCore import QVariant
from qgis.PyQt.QtGui import QColor
from qgis.core import (
    QgsCategorizedSymbolRenderer,
    QgsFeature,
    QgsFeatureSink,
    QgsField,
    QgsFields,
    QgsGeometry,
    QgsProcessing,
    QgsProcessingAlgorithm,
    QgsProcessingException,
    QgsProcessingParameterBoolean,
    QgsProcessingParameterFeatureSink,
    QgsProcessingParameterFeatureSource,
    QgsProcessingUtils,
    QgsRendererCategory,
    QgsSymbol,
    QgsWkbTypes,
)

from . import tissot


class TissotDiffAlgorithm(QgsProcessingAlgorithm):
    """Run Tissot spatial diff between two vector layers."""

    INPUT_BASELINE = "INPUT_BASELINE"
    INPUT_COMPARISON = "INPUT_COMPARISON"
    OPEN_REPORT = "OPEN_REPORT"
    OUTPUT_CHANGES = "OUTPUT_CHANGES"

    def name(self):
        return "diff"

    def displayName(self):
        return self.tr("Spatial Diff")

    def group(self):
        return self.tr("Tissot")

    def groupId(self):
        return "tissot"

    def shortHelpString(self):
        return self.tr(
            "Runs tissot.diff between baseline and comparison layers and "
            "outputs a change layer categorized by change type."
        )

    def initAlgorithm(self, config=None):
        self.addParameter(
            QgsProcessingParameterFeatureSource(
                self.INPUT_BASELINE,
                self.tr("Baseline layer"),
                [QgsProcessing.TypeVectorAnyGeometry],
            )
        )
        self.addParameter(
            QgsProcessingParameterFeatureSource(
                self.INPUT_COMPARISON,
                self.tr("Comparison layer"),
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
                self.OUTPUT_CHANGES,
                self.tr("Change layer"),
                type=QgsProcessing.TypeVectorAnyGeometry,
            )
        )

    def processAlgorithm(self, parameters, context, feedback):
        baseline_source = self.parameterAsSource(parameters, self.INPUT_BASELINE, context)
        comparison_source = self.parameterAsSource(parameters, self.INPUT_COMPARISON, context)
        open_report = self.parameterAsBool(parameters, self.OPEN_REPORT, context)

        if baseline_source is None or comparison_source is None:
            raise QgsProcessingException(self.tr("Invalid input layers."))

        feedback.pushInfo(self.tr("Preparing input data..."))
        baseline_path = self.parameterAsCompatibleSourceLayerPath(
            parameters, self.INPUT_BASELINE, context,
            compatibleFormats=['geojson'], preferredFormat='geojson',
            feedback=feedback,
        )
        comparison_path = self.parameterAsCompatibleSourceLayerPath(
            parameters, self.INPUT_COMPARISON, context,
            compatibleFormats=['geojson'], preferredFormat='geojson',
            feedback=feedback,
        )
        feedback.setProgress(20)

        try:
            feedback.pushInfo(self.tr("Running tissot.diff..."))
            report = tissot.diff(baseline_path, comparison_path)
        except Exception as exc:  # pylint: disable=broad-exception-caught
            feedback.reportError(f"tissot.diff failed: {exc}", fatalError=True)
            raise QgsProcessingException(str(exc)) from exc

        changes = self._safe_get(report, "changes", default=[]) or []
        feedback.setProgress(45)
        baseline_index = self._build_feature_index(baseline_source)
        comparison_index = self._build_feature_index(comparison_source)

        fields = QgsFields()
        fields.append(QgsField("change_type", QVariant.String))
        fields.append(QgsField("feature_id", QVariant.String))
        fields.append(QgsField("message", QVariant.String))

        sink, dest_id = self.parameterAsSink(
            parameters,
            self.OUTPUT_CHANGES,
            context,
            fields,
            QgsWkbTypes.Unknown,
            baseline_source.sourceCrs(),
        )
        if sink is None:
            raise QgsProcessingException(self.tr("Could not create output sink."))

        total = max(len(changes), 1)
        emitted = 0
        for index, change in enumerate(changes):
            if feedback.isCanceled():
                break
            feature = QgsFeature(fields)
            change_type = self._safe_get(change, "change_type", "type", default="modified")
            feature.setAttribute("change_type", str(change_type))
            feature_id = str(self._safe_get(change, "feature_id", "id", default=""))
            feature.setAttribute("feature_id", feature_id)
            feature.setAttribute("message", self._safe_get(change, "message", default=""))

            geometry = self._coerce_geometry(self._safe_get(change, "geometry", "geom"))
            if geometry is None:
                geometry = self._fallback_geometry(
                    str(change_type),
                    feature_id,
                    baseline_index,
                    comparison_index,
                )

            if geometry is None:
                continue

            feature.setGeometry(geometry)

            sink.addFeature(feature, QgsFeatureSink.FastInsert)
            emitted += 1
            feedback.setProgress(45 + int(((index + 1) / total) * 45))

        feedback.pushInfo(self.tr(f"Exported {emitted} diff findings."))

        self._style_change_layer(dest_id, context)

        report_ref = self._safe_get(report, "report_url", "report_path", "html_report")
        if open_report and report_ref:
            self._open_report(report_ref, feedback)

        feedback.setProgress(100)
        return {self.OUTPUT_CHANGES: dest_id}

    def createInstance(self):
        return TissotDiffAlgorithm()

    def _safe_get(self, value, *names, default=None):
        for name in names:
            if isinstance(value, dict) and name in value:
                return value[name]
            if hasattr(value, name):
                return getattr(value, name)
        return default

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

        if hasattr(raw_geometry, "isNull"):
            return raw_geometry if not raw_geometry.isNull() else None

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

    def _fallback_geometry(self, change_type, feature_id, baseline_index, comparison_index):
        if not feature_id:
            return None

        if change_type == "added":
            return comparison_index.get(feature_id)
        if change_type == "removed":
            return baseline_index.get(feature_id)
        return comparison_index.get(feature_id) or baseline_index.get(feature_id)

    def _style_change_layer(self, destination_id, context):
        result_layer = QgsProcessingUtils.mapLayerFromString(destination_id, context)
        if result_layer is None:
            return

        categories = []
        for value, label, color_hex in [
            ("added", "Added", "#22c55e"),
            ("removed", "Removed", "#ef4444"),
            ("modified", "Modified", "#f59e0b"),
        ]:
            symbol = QgsSymbol.defaultSymbol(result_layer.geometryType())
            symbol.setColor(QColor(color_hex))
            categories.append(QgsRendererCategory(value, symbol, label))

        renderer = QgsCategorizedSymbolRenderer("change_type", categories)
        result_layer.setRenderer(renderer)
        result_layer.triggerRepaint()

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
