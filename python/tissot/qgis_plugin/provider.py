"""QGIS Processing provider for Tissot algorithms."""

import os

from qgis.PyQt.QtGui import QIcon
from qgis.core import QgsApplication, QgsProcessingProvider

from .check_algorithm import TissotCheckAlgorithm
from .diff_algorithm import TissotDiffAlgorithm
from .score_algorithm import TissotScoreAlgorithm
from .xray_algorithm import TissotXrayAlgorithm


class TissotProvider(QgsProcessingProvider):
    """Processing provider that exposes Tissot algorithms."""

    def id(self):
        return "tissot"

    def name(self):
        return self.tr("Tissot")

    def longName(self):
        return self.name()

    def icon(self):
        icon_path = os.path.join(os.path.dirname(__file__), "icon.png")
        if not os.path.exists(icon_path):
            icon_path = os.path.join(os.path.dirname(__file__), "icon-qgis.svg")
        return QIcon(icon_path)

    def loadAlgorithms(self):
        self.addAlgorithm(TissotXrayAlgorithm())
        self.addAlgorithm(TissotCheckAlgorithm())
        self.addAlgorithm(TissotScoreAlgorithm())
        self.addAlgorithm(TissotDiffAlgorithm())


class TissotPlugin:
    """Main QGIS plugin wrapper that registers the processing provider."""

    def __init__(self, iface):
        self.iface = iface
        self.provider = None

    def initProcessing(self):
        self.provider = TissotProvider()
        QgsApplication.processingRegistry().addProvider(self.provider)

    def initGui(self):
        self.initProcessing()

    def unload(self):
        if self.provider:
            QgsApplication.processingRegistry().removeProvider(self.provider)
            self.provider = None
