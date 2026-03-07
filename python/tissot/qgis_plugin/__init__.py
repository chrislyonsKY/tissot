"""QGIS plugin entrypoint for Tissot Processing provider."""

from .provider import TissotPlugin


def classFactory(iface):  # pylint: disable=invalid-name
    """Instantiate the QGIS plugin class."""
    return TissotPlugin(iface)
