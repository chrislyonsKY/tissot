# Tissot Architecture Diagram

```mermaid
flowchart TB
    CLI[CLI\ntissot xray/check/diff/fix/score/watch]

    IO[IO Layer\ngeojson/shapefile/flatgeobuf/geopackage]
    CHECK[Checker Engine\nRule registry + findings]
    XRAY[X-Ray Engine\nDistortion + recommendations]
    FIX[Fix Engine\nReproject + topology heal]
    SCORE[Score Engine\n0-100 category scoring]
    VIS[Visual Report Server\naxum + self-contained HTML]

    CLI --> IO
    CLI --> CHECK
    CLI --> XRAY
    CLI --> FIX
    CLI --> SCORE

    IO --> CHECK
    IO --> XRAY
    CHECK --> SCORE
    CHECK --> VIS
    XRAY --> VIS
    SCORE --> VIS

    VIS --> OUT1[Interactive Map Reports]
    CHECK --> OUT2[SARIF/JSON/Terminal]
    FIX --> OUT3[Fixed GeoJSON outputs]
```

The architecture is intentionally visual-first: findings and distortion metrics are spatially rendered for rapid diagnostics, while machine-readable outputs are provided for CI/CD.
