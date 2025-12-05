// Live integration test for the Geonorge address API via GeoNorgeClient.
// This test hits the real API and may be flaky due to network; it's ignored by default.

#[cfg(test)]
mod integration {
    use lib_clients::geonorge::GeoNorgeClient;

    // Run with: cargo test -p lib-clients -- --ignored
    #[ignore]
    #[tokio::test]
    async fn returns_koordinater_for_known_address() {
        let client = GeoNorgeClient::new();

        let coords = client
            .get_koordinater("Karl Johans gate 1, Oslo")
            .await
            .expect("request to geonorge should succeed");

        let (lat, lon) = coords.expect("should find at least one address with koordinater");

        // Print koordinater so they are visible when running with `-- --nocapture`
        println!("koordinater for 'Karl Johans gate 1, Oslo': lat={lat}, lon={lon}");

        // Assert koordinater are within Norway's rough bounding box
        // Latitude: 57.9 ..= 71.3, Longitude: 4.5 ..= 31.7
        assert!(
            (57.9..=71.3).contains(&lat),
            "lat out of Norway bounds: {lat}"
        );
        assert!(
            (4.5..=31.7).contains(&lon),
            "lon out of Norway bounds: {lon}"
        );
    }
}
