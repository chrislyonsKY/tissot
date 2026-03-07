"""QGIS Processing algorithm wrapper for tissot.score."""

from pathlib import Path
import webbrowser

from qgis.core import (
    QgsProcessing,
    QgsProcessingAlgorithm,
    QgsProcessingException,
    QgsProcessingOutputNumber,
    QgsProcessingOutputString,
    QgsProcessingParameterBoolean,
    QgsProcessingParameterFeatureSource,
    QgsProcessingParameterFileDestination,
)

from . import tissot


class TissotScoreAlgorithm(QgsProcessingAlgorithm):
    """Run Tissot map quality scoring for an input vector layer."""

    INPUT = "INPUT"
    OPEN_REPORT = "OPEN_REPORT"
    BADGE_OUTPUT = "BADGE_OUTPUT"
    OUTPUT_SCORE = "OUTPUT_SCORE"
    OUTPUT_CATEGORIES = "OUTPUT_CATEGORIES"

    def name(self):
        return "score"

    def displayName(self):
        return self.tr("Map Quality Score")

    def group(self):
        return self.tr("Tissot")

    def groupId(self):
        return "tissot"

    def shortHelpString(self):
        return self.tr(
            "Runs tissot.score on an input layer and returns an overall score, "
            "category breakdown text, and optional badge SVG."
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
            QgsProcessingParameterFileDestination(
                self.BADGE_OUTPUT,
                self.tr("Optional SVG badge output"),
                fileFilter="SVG files (*.svg)",
                optional=True,
            )
        )

        self.addOutput(
            QgsProcessingOutputNumber(
                self.OUTPUT_SCORE,
                self.tr("Overall score"),
            )
        )
        self.addOutput(
            QgsProcessingOutputString(
                self.OUTPUT_CATEGORIES,
                self.tr("Category breakdown"),
            )
        )

    def processAlgorithm(self, parameters, context, feedback):
        source = self.parameterAsSource(parameters, self.INPUT, context)
        open_report = self.parameterAsBool(parameters, self.OPEN_REPORT, context)
        badge_output = self.parameterAsFileOutput(parameters, self.BADGE_OUTPUT, context)

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
            feedback.pushInfo(self.tr("Running tissot.score..."))
            report = self._run_score(input_path, badge_output)
        except Exception as exc:  # pylint: disable=broad-exception-caught
            feedback.reportError(f"tissot.score failed: {exc}", fatalError=True)
            raise QgsProcessingException(str(exc)) from exc

        score = float(self._safe_get(report, "score", default=0.0) or 0.0)
        categories = self._safe_get(report, "categories", "category_breakdown", default={})
        category_text = self._format_categories(categories)
        feedback.setProgress(70)

        report_ref = self._safe_get(report, "report_url", "report_path", "html_report")
        if open_report and report_ref:
            self._open_report(report_ref, feedback)

        feedback.setProgress(100)
        return {
            self.OUTPUT_SCORE: score,
            self.OUTPUT_CATEGORIES: category_text,
            self.BADGE_OUTPUT: badge_output,
        }

    def createInstance(self):
        return TissotScoreAlgorithm()

    def _safe_get(self, value, *names, default=None):
        for name in names:
            if isinstance(value, dict) and name in value:
                return value[name]
            if hasattr(value, name):
                return getattr(value, name)
        return default

    def _format_categories(self, categories):
        if categories is None:
            return ""
        if isinstance(categories, dict):
            parts = []
            for key, value in categories.items():
                parts.append(f"{key}: {value}")
            return "; ".join(parts)
        if isinstance(categories, list):
            return "; ".join(str(item) for item in categories)
        return str(categories)

    def _run_score(self, input_path, badge_output):
        score_fn = getattr(tissot, "score", None)
        if score_fn is not None:
            if badge_output:
                try:
                    return score_fn(input_path, badge=badge_output)
                except TypeError:
                    return score_fn(input_path)
            return score_fn(input_path)

        # Compatibility fallback for older bindings where score is available on check report.
        return tissot.check(input_path)

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
