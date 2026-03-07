"""QGIS Processing algorithm wrapper for tissot.fix."""

from pathlib import Path

from qgis.PyQt.QtCore import QCoreApplication, QVariant
from qgis.core import (
    QgsCoordinateReferenceSystem,
    QgsFeature,
    QgsFeatureSink,
    QgsField,
    QgsFields,
    QgsProcessing,
    QgsProcessingAlgorithm,
    QgsProcessingException,
    QgsProcessingOutputNumber,
    QgsProcessingOutputString,
    QgsProcessingParameterBoolean,
    QgsProcessingParameterCrs,
    QgsProcessingParameterFeatureSink,
    QgsProcessingParameterFeatureSource,
    QgsWkbTypes,
)

from . import tissot


class TissotFixAlgorithm(QgsProcessingAlgorithm):
    """Run Tissot autofix (reproject / topology healing) on an input layer."""

    INPUT = "INPUT"
    REPROJECT = "REPROJECT"
    TARGET_CRS = "TARGET_CRS"
    TOPOLOGY = "TOPOLOGY"
    OUTPUT = "OUTPUT"
    OUTPUT_UPDATED = "OUTPUT_UPDATED"
    OUTPUT_ACTIONS = "OUTPUT_ACTIONS"

    @staticmethod
    def tr(message):
        return QCoreApplication.translate("TissotFixAlgorithm", message)

    def name(self):
        return "fix"

    def displayName(self):
        return self.tr("Autofix")

    def group(self):
        return self.tr("Tissot")

    def groupId(self):
        return "tissot"

    def shortHelpString(self):
        return self.tr(
            "Runs tissot.fix on an input layer to reproject to an optimal CRS "
            "and/or heal topology issues (null geometries, duplicates). "
            "The fixed layer is returned as a new output."
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
                self.REPROJECT,
                self.tr("Reproject to target CRS"),
                defaultValue=False,
            )
        )
        self.addParameter(
            QgsProcessingParameterCrs(
                self.TARGET_CRS,
                self.tr("Target CRS (used when Reproject is enabled)"),
                optional=True,
            )
        )
        self.addParameter(
            QgsProcessingParameterBoolean(
                self.TOPOLOGY,
                self.tr("Heal topology (remove null/duplicate geometries)"),
                defaultValue=False,
            )
        )
        self.addParameter(
            QgsProcessingParameterFeatureSink(
                self.OUTPUT,
                self.tr("Fixed layer"),
                type=QgsProcessing.TypeVectorAnyGeometry,
            )
        )
        self.addOutput(
            QgsProcessingOutputNumber(
                self.OUTPUT_UPDATED,
                self.tr("Updated feature count"),
            )
        )
        self.addOutput(
            QgsProcessingOutputString(
                self.OUTPUT_ACTIONS,
                self.tr("Actions performed"),
            )
        )

    def processAlgorithm(self, parameters, context, feedback):
        source = self.parameterAsSource(parameters, self.INPUT, context)
        do_reproject = self.parameterAsBool(parameters, self.REPROJECT, context)
        do_topology = self.parameterAsBool(parameters, self.TOPOLOGY, context)

        if source is None:
            raise QgsProcessingException(self.tr("Invalid input layer."))

        if not do_reproject and not do_topology:
            raise QgsProcessingException(
                self.tr("Select at least one fix: Reproject or Topology.")
            )

        target_crs_str = None
        if do_reproject:
            crs_param = self.parameterAsCrs(parameters, self.TARGET_CRS, context)
            if crs_param is None or not crs_param.isValid():
                raise QgsProcessingException(
                    self.tr("A valid Target CRS is required when Reproject is enabled.")
                )
            target_crs_str = crs_param.authid()

        feedback.pushInfo(self.tr("Preparing input data..."))
        input_path = self.parameterAsCompatibleSourceLayerPath(
            parameters, self.INPUT, context,
            compatibleFormats=["geojson"], preferredFormat="geojson",
            feedback=feedback,
        )
        feedback.setProgress(15)

        try:
            feedback.pushInfo(self.tr("Running tissot.fix..."))
            report = tissot.fix(
                input_path,
                reproject=target_crs_str if do_reproject else None,
                topology=do_topology,
            )
        except Exception as exc:
            feedback.reportError(f"tissot.fix failed: {exc}", fatalError=True)
            raise QgsProcessingException(str(exc)) from exc

        feedback.setProgress(55)

        output_path = report.get("output", "")
        updated_features = int(report.get("updated_features", 0))
        actions = report.get("actions", [])

        if not output_path or not Path(output_path).exists():
            raise QgsProcessingException(
                self.tr("Fix produced no output file.")
            )

        fixed_layers = self._read_geojson_features(output_path)

        out_crs = (
            QgsCoordinateReferenceSystem(target_crs_str)
            if target_crs_str
            else source.sourceCrs()
        )

        fields = QgsFields()
        for field in source.fields():
            fields.append(field)
        if fields.isEmpty():
            fields.append(QgsField("fid", QVariant.String))

        sink, dest_id = self.parameterAsSink(
            parameters, self.OUTPUT, context,
            fields, QgsWkbTypes.Unknown, out_crs,
        )
        if sink is None:
            raise QgsProcessingException(self.tr("Could not create output sink."))

        total = max(len(fixed_layers), 1)
        for index, gjson_feat in enumerate(fixed_layers):
            if feedback.isCanceled():
                break

            from qgis.core import QgsGeometry, QgsJsonUtils
            geom_dict = gjson_feat.get("geometry")
            geometry = None
            if geom_dict:
                import json as _json
                geometry = QgsGeometry.fromWkt(
                    QgsGeometry.fromJson(_json.dumps(geom_dict)).asWkt()
                ) if geom_dict else None

            feature = QgsFeature(fields)
            if geometry and not geometry.isNull():
                feature.setGeometry(geometry)

            props = gjson_feat.get("properties") or {}
            for i in range(fields.count()):
                fname = fields.at(i).name()
                if fname in props:
                    feature.setAttribute(fname, props[fname])

            sink.addFeature(feature, QgsFeatureSink.FastInsert)
            feedback.setProgress(55 + int(((index + 1) / total) * 40))

        actions_str = "; ".join(str(a) for a in actions)
        feedback.pushInfo(
            self.tr(f"Fix complete: {updated_features} features updated. Actions: {actions_str}")
        )
        feedback.setProgress(100)

        return {
            self.OUTPUT: dest_id,
            self.OUTPUT_UPDATED: updated_features,
            self.OUTPUT_ACTIONS: actions_str,
        }

    def createInstance(self):
        return TissotFixAlgorithm()

    @staticmethod
    def _read_geojson_features(path: str) -> list[dict]:
        """Read features from a GeoJSON file produced by tissot fix."""
        import json as _json
        text = Path(path).read_text(encoding="utf-8")
        data = _json.loads(text)
        if isinstance(data, dict):
            return data.get("features", [])
        return []
