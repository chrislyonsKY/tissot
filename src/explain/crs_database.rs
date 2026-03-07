//! Curated lookup table of common EPSG codes with human-readable descriptions.
//!
//! Contains 20+ entries covering WGS84, Web Mercator, US State Plane,
//! UTM zones, continental equal-area projections, and more.

use serde::{Deserialize, Serialize};

/// A curated entry for a known EPSG coordinate reference system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrsEntry {
    /// EPSG code.
    pub epsg: u32,
    /// Short name (e.g., "WGS 84", "Web Mercator").
    pub name: String,
    /// Plain-English description of the CRS and its purpose.
    pub description: String,
    /// Whether the CRS preserves area (equal-area projection).
    pub preserves_area: bool,
    /// Whether the CRS preserves shape (conformal projection).
    pub preserves_shape: bool,
    /// Whether the CRS preserves distance along certain lines.
    pub preserves_distance: bool,
    /// Whether the CRS preserves direction (azimuthal).
    pub preserves_direction: bool,
    /// What the CRS is suitable for.
    pub suitable_for: String,
    /// Geographic region covered.
    pub region: String,
}

/// Look up a CRS entry by EPSG code.
///
/// Returns `Some(CrsEntry)` if the code is in the curated database,
/// `None` otherwise.
pub fn lookup(epsg: u32) -> Option<CrsEntry> {
    database().into_iter().find(|e| e.epsg == epsg)
}

/// The curated EPSG database.
fn database() -> Vec<CrsEntry> {
    vec![
        CrsEntry {
            epsg: 4326,
            name: "WGS 84".into(),
            description: "World Geodetic System 1984 — the GPS coordinate system. Geographic (lat/lon) coordinates on the WGS84 ellipsoid.".into(),
            preserves_area: false,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "GPS data, worldwide reference, data exchange".into(),
            region: "World".into(),
        },
        CrsEntry {
            epsg: 3857,
            name: "Web Mercator".into(),
            description: "Pseudo-Mercator projection used by web mapping services (Google Maps, OpenStreetMap). Severely distorts area at high latitudes.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: true,
            suitable_for: "Web maps, navigation, tile-based mapping".into(),
            region: "World (excl. poles)".into(),
        },
        CrsEntry {
            epsg: 3089,
            name: "KY Single Zone (NAD83)".into(),
            description: "Kentucky Single Zone State Plane — Lambert Conformal Conic optimized for Kentucky.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Statewide mapping in Kentucky, cadastral, engineering".into(),
            region: "Kentucky, USA".into(),
        },
        CrsEntry {
            epsg: 2205,
            name: "KY FIPS 1600 (NAD83, ft)".into(),
            description: "Kentucky FIPS Zone 1600 — single zone State Plane in US survey feet.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Kentucky engineering and surveying in US feet".into(),
            region: "Kentucky, USA".into(),
        },
        CrsEntry {
            epsg: 32617,
            name: "UTM Zone 17N (WGS84)".into(),
            description: "Universal Transverse Mercator zone 17 North — covers 84°W to 78°W.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Topographic mapping, medium-scale regional work".into(),
            region: "Eastern USA, Caribbean (78°W–84°W)".into(),
        },
        CrsEntry {
            epsg: 32618,
            name: "UTM Zone 18N (WGS84)".into(),
            description: "Universal Transverse Mercator zone 18 North — covers 78°W to 72°W.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Topographic mapping, medium-scale regional work".into(),
            region: "Mid-Atlantic USA (72°W–78°W)".into(),
        },
        CrsEntry {
            epsg: 32619,
            name: "UTM Zone 19N (WGS84)".into(),
            description: "Universal Transverse Mercator zone 19 North — covers 72°W to 66°W.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Topographic mapping, medium-scale regional work".into(),
            region: "New England USA, Maritime Canada (66°W–72°W)".into(),
        },
        CrsEntry {
            epsg: 5070,
            name: "NAD83 / Conus Albers".into(),
            description: "Albers Equal-Area Conic for the contiguous United States. Standard parallels at 29.5°N and 45.5°N.".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Area calculations, thematic mapping, CONUS-wide analysis".into(),
            region: "Contiguous United States".into(),
        },
        CrsEntry {
            epsg: 6933,
            name: "World Cylindrical Equal Area".into(),
            description: "Cylindrical Equal-Area projection for global equal-area analysis.".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Global area calculations, density mapping".into(),
            region: "World".into(),
        },
        CrsEntry {
            epsg: 3035,
            name: "ETRS89 / LAEA Europe".into(),
            description: "Lambert Azimuthal Equal-Area projection centered on Europe (52°N, 10°E).".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "European statistical mapping, EU INSPIRE compliance".into(),
            region: "Europe".into(),
        },
        CrsEntry {
            epsg: 4269,
            name: "NAD83".into(),
            description: "North American Datum 1983 — geographic coordinates for North America. Very close to WGS84 but based on GRS80 ellipsoid.".into(),
            preserves_area: false,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "North American geographic coordinates, datum for State Plane systems".into(),
            region: "North America".into(),
        },
        CrsEntry {
            epsg: 4267,
            name: "NAD27".into(),
            description: "North American Datum 1927 — legacy datum based on the Clarke 1866 ellipsoid. Offsets from NAD83/WGS84 can exceed 100 meters.".into(),
            preserves_area: false,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Legacy data compatibility only — convert to NAD83 for modern use".into(),
            region: "North America (historical)".into(),
        },
        CrsEntry {
            epsg: 26917,
            name: "NAD83 / UTM Zone 17N".into(),
            description: "UTM zone 17N on the NAD83 datum. Equivalent to EPSG:32617 but datum-specific.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Regional mapping in eastern USA on NAD83 datum".into(),
            region: "Eastern USA (78°W–84°W)".into(),
        },
        CrsEntry {
            epsg: 26918,
            name: "NAD83 / UTM Zone 18N".into(),
            description: "UTM zone 18N on the NAD83 datum.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Regional mapping in mid-Atlantic USA on NAD83 datum".into(),
            region: "Mid-Atlantic USA (72°W–78°W)".into(),
        },
        CrsEntry {
            epsg: 26919,
            name: "NAD83 / UTM Zone 19N".into(),
            description: "UTM zone 19N on the NAD83 datum.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Regional mapping in New England USA on NAD83 datum".into(),
            region: "New England USA (66°W–72°W)".into(),
        },
        CrsEntry {
            epsg: 2163,
            name: "US National Atlas Equal Area".into(),
            description: "Lambert Azimuthal Equal-Area projection centered on the US (45°N, 100°W). Used for national-scale thematic maps.".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "US national-scale thematic and density maps".into(),
            region: "United States".into(),
        },
        CrsEntry {
            epsg: 102003,
            name: "Albers Equal Area (North America)".into(),
            description: "Albers Equal-Area Conic for North America. Custom ESRI code (not official EPSG). Standard parallels at 20°N and 60°N.".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Continental-scale area analysis across North America".into(),
            region: "North America".into(),
        },
        CrsEntry {
            epsg: 54030,
            name: "Robinson".into(),
            description: "Robinson projection — compromise projection that looks 'right' without preserving any metric property exactly. Used by National Geographic.".into(),
            preserves_area: false,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "World wall maps, general reference, aesthetic global views".into(),
            region: "World".into(),
        },
        CrsEntry {
            epsg: 3338,
            name: "NAD83 / Alaska Albers".into(),
            description: "Albers Equal-Area Conic optimized for Alaska. Standard parallels at 55°N and 65°N.".into(),
            preserves_area: true,
            preserves_shape: false,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Alaska statewide mapping, area calculations".into(),
            region: "Alaska, USA".into(),
        },
        CrsEntry {
            epsg: 32601,
            name: "UTM Zone 1N (WGS84)".into(),
            description: "Universal Transverse Mercator zone 1 North — covers 180°W to 174°W.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Topographic mapping in far western Pacific".into(),
            region: "Western Pacific (174°W–180°W)".into(),
        },
        CrsEntry {
            epsg: 2278,
            name: "NAD83 / Texas Central (ftUS)".into(),
            description: "Texas Central Zone State Plane in US survey feet. Lambert Conformal Conic.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Central Texas surveying and engineering in US feet".into(),
            region: "Central Texas, USA".into(),
        },
        CrsEntry {
            epsg: 3826,
            name: "TWD97 / TM2 zone 121".into(),
            description: "Taiwan Datum 1997 — Transverse Mercator for Taiwan.".into(),
            preserves_area: false,
            preserves_shape: true,
            preserves_distance: false,
            preserves_direction: false,
            suitable_for: "Mapping and surveying in Taiwan".into(),
            region: "Taiwan".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_wgs84() {
        let entry = lookup(4326).unwrap();
        assert_eq!(entry.name, "WGS 84");
        assert_eq!(entry.region, "World");
    }

    #[test]
    fn lookup_web_mercator() {
        let entry = lookup(3857).unwrap();
        assert_eq!(entry.name, "Web Mercator");
        assert!(entry.preserves_shape);
        assert!(!entry.preserves_area);
    }

    #[test]
    fn lookup_albers_conus() {
        let entry = lookup(5070).unwrap();
        assert!(entry.preserves_area);
        assert!(!entry.preserves_shape);
        assert!(entry.suitable_for.contains("Area"));
    }

    #[test]
    fn lookup_unknown() {
        assert!(lookup(99999).is_none());
    }

    #[test]
    fn database_has_at_least_20_entries() {
        assert!(database().len() >= 20);
    }

    #[test]
    fn all_entries_have_descriptions() {
        for entry in database() {
            assert!(!entry.name.is_empty(), "EPSG:{} has empty name", entry.epsg);
            assert!(
                !entry.description.is_empty(),
                "EPSG:{} has empty description",
                entry.epsg
            );
        }
    }
}
