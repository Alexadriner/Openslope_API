use crate::models::db::Place;
use serde_json::Value;
use std::collections::HashSet;

pub fn parse_feature_id(feature: &Value) -> Option<String> {
    if let Some(id) = feature.get("id") {
        extract_string(id)
    } else {
        feature
            .get("properties")
            .and_then(|props| props.get("id"))
            .and_then(extract_string)
    }
}

pub fn parse_related_ids(properties: &Value) -> Vec<String> {
    let ski_areas = properties
        .get("skiAreas")
        .or_else(|| properties.get("ski_areas"));
    match ski_areas {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                if let Some(value) = extract_string(item) {
                    return Some(value);
                }
                item.get("id").and_then(extract_string).or_else(|| {
                    item.get("properties")
                        .and_then(|inner| inner.get("id"))
                        .and_then(extract_string)
                })
            })
            .collect(),
        Some(Value::String(id)) => vec![id.clone()],
        _ => Vec::new(),
    }
}

pub fn parse_places(properties: &Value) -> Vec<Place> {
    let Some(Value::Array(raw_places)) = properties.get("places") else {
        return Vec::new();
    };

    let mut seen = HashSet::new();
    let mut places = Vec::new();

    for raw_place in raw_places {
        let Some(place) = parse_place(raw_place) else {
            continue;
        };

        let key = (
            place.country_code.clone(),
            place.region_code.clone(),
            place.locality.clone(),
        );

        if seen.insert(key) {
            places.push(place);
        }
    }

    places
}

pub fn parse_geometry_to_wkt(geometry: &Value) -> Option<String> {
    let geometry_type = geometry.get("type")?.as_str()?.to_uppercase();
    let coords = geometry.get("coordinates")?;

    // Try to parse the geometry. If it fails, return None (will be treated as invalid)
    match geometry_type.as_str() {
        "POINT" => parse_point(coords).map(|(x, y)| format!("POINT({} {})", x, y)),
        "LINESTRING" => parse_line(coords).map(|points| {
            let coordinates = points
                .into_iter()
                .map(|(x, y)| format!("{} {}", x, y))
                .collect::<Vec<_>>()
                .join(", ");
            format!("LINESTRING({})", coordinates)
        }),
        "POLYGON" => parse_polygon(coords).and_then(|rings| {
            // Validate and close the polygon rings
            let mut valid_rings = Vec::new();
            for ring in rings {
                if ring.len() < 3 {
                    // Ring has less than 3 points, invalid
                    return None;
                }

                // Ensure ring is closed: first and last points must be identical
                let mut closed_ring = ring;
                let first = closed_ring[0];
                let last = closed_ring[closed_ring.len() - 1];
                if first != last {
                    closed_ring.push(first);
                }
                valid_rings.push(closed_ring);
            }

            if valid_rings.is_empty() {
                return None;
            }

            let rings_wkt = valid_rings
                .into_iter()
                .map(|ring| {
                    let points = ring
                        .into_iter()
                        .map(|(x, y)| format!("{} {}", x, y))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({})", points)
                })
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("POLYGON({})", rings_wkt))
        }),
        "MULTILINESTRING" => parse_multi_line(coords).map(|lines| {
            let lines = lines
                .into_iter()
                .map(|line| {
                    let points = line
                        .into_iter()
                        .map(|(x, y)| format!("{} {}", x, y))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({})", points)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("MULTILINESTRING({})", lines)
        }),
        "MULTIPOLYGON" => parse_multi_polygon(coords).and_then(|polygons| {
            let mut valid_polygons = Vec::new();

            for rings in polygons {
                let mut valid_rings = Vec::new();
                for ring in rings {
                    if ring.len() < 3 {
                        continue; // Skip invalid rings
                    }

                    let mut closed_ring = ring;
                    let first = closed_ring[0];
                    let last = closed_ring[closed_ring.len() - 1];
                    if first != last {
                        closed_ring.push(first);
                    }
                    valid_rings.push(closed_ring);
                }

                if !valid_rings.is_empty() {
                    valid_polygons.push(valid_rings);
                }
            }

            if valid_polygons.is_empty() {
                return None;
            }

            let polygons_wkt = valid_polygons
                .into_iter()
                .map(|rings| {
                    let rings_text = rings
                        .into_iter()
                        .map(|ring| {
                            let points = ring
                                .into_iter()
                                .map(|(x, y)| format!("{} {}", x, y))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("({})", points)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({})", rings_text)
                })
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("MULTIPOLYGON({})", polygons_wkt))
        }),
        _ => None,
    }
}

pub fn parse_geometry_to_linestring_wkt(geometry: &Value) -> Option<String> {
    let geometry_type = geometry.get("type")?.as_str()?.to_uppercase();
    let coords = geometry.get("coordinates")?;

    let points = match geometry_type.as_str() {
        "LINESTRING" => sanitize_line(parse_line(coords)?)?,
        "MULTILINESTRING" => parse_multi_line(coords)?
            .into_iter()
            .filter_map(sanitize_line)
            .max_by(|a, b| line_length(a).total_cmp(&line_length(b)))?,
        "POLYGON" => parse_polygon(coords)?
            .into_iter()
            .next()
            .and_then(sanitize_ring_as_line)?,
        "MULTIPOLYGON" => parse_multi_polygon(coords)?
            .into_iter()
            .filter_map(|polygon| polygon.into_iter().next())
            .filter_map(sanitize_ring_as_line)
            .max_by(|a, b| line_length(a).total_cmp(&line_length(b)))?,
        _ => return None,
    };

    Some(format!(
        "LINESTRING({})",
        points
            .into_iter()
            .map(|(x, y)| format!("{} {}", x, y))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn extract_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(num) => Some(num.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn parse_place(value: &Value) -> Option<Place> {
    let localized_en = value
        .get("localized")
        .and_then(|localized| localized.get("en"));

    let place = Place {
        id: 0,
        country_code: value.get("iso3166_1Alpha2").and_then(extract_string),
        region_code: value.get("iso3166_2").and_then(extract_string),
        country_name: localized_en
            .and_then(|localized| localized.get("country"))
            .and_then(extract_string),
        region_name: localized_en
            .and_then(|localized| localized.get("region"))
            .and_then(extract_string),
        locality: localized_en
            .and_then(|localized| localized.get("locality"))
            .and_then(extract_string),
    };

    if place.country_code.is_none()
        && place.region_code.is_none()
        && place.country_name.is_none()
        && place.region_name.is_none()
        && place.locality.is_none()
    {
        None
    } else {
        Some(place)
    }
}

fn parse_point(coords: &Value) -> Option<(f64, f64)> {
    if let Value::Array(arr) = coords {
        if arr.len() >= 2 {
            let x = arr[0].as_f64();
            let y = arr[1].as_f64();
            return x.and_then(|x| y.map(|y| (x, y)));
        }
    }
    None
}

fn parse_line(coords: &Value) -> Option<Vec<(f64, f64)>> {
    if let Value::Array(arr) = coords {
        let mut points = Vec::new();
        for item in arr {
            if let Some(point) = parse_point(item) {
                points.push(point);
            }
        }
        if !points.is_empty() {
            return Some(points);
        }
    }
    None
}

fn parse_polygon(coords: &Value) -> Option<Vec<Vec<(f64, f64)>>> {
    if let Value::Array(rings) = coords {
        let mut polygon = Vec::new();
        for ring in rings {
            if let Some(points) = parse_line(ring) {
                polygon.push(points);
            }
        }
        if !polygon.is_empty() {
            return Some(polygon);
        }
    }
    None
}

fn parse_multi_line(coords: &Value) -> Option<Vec<Vec<(f64, f64)>>> {
    if let Value::Array(lines) = coords {
        let mut output = Vec::new();
        for line in lines {
            if let Some(points) = parse_line(line) {
                output.push(points);
            }
        }
        if !output.is_empty() {
            return Some(output);
        }
    }
    None
}

fn parse_multi_polygon(coords: &Value) -> Option<Vec<Vec<Vec<(f64, f64)>>>> {
    if let Value::Array(polygons) = coords {
        let mut output = Vec::new();
        for polygon in polygons {
            if let Some(rings) = parse_polygon(polygon) {
                output.push(rings);
            }
        }
        if !output.is_empty() {
            return Some(output);
        }
    }
    None
}

fn sanitize_line(points: Vec<(f64, f64)>) -> Option<Vec<(f64, f64)>> {
    let deduped = dedupe_consecutive_points(points);
    if distinct_point_count(&deduped) < 2 {
        return None;
    }
    Some(deduped)
}

fn sanitize_ring_as_line(mut points: Vec<(f64, f64)>) -> Option<Vec<(f64, f64)>> {
    if points.len() < 3 {
        return None;
    }

    if points.first() != points.last() {
        let first = points[0];
        points.push(first);
    }

    let deduped = dedupe_consecutive_points(points);
    if distinct_point_count(&deduped) < 3 || deduped.len() < 4 {
        return None;
    }

    Some(deduped)
}

fn dedupe_consecutive_points(points: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let mut deduped = Vec::with_capacity(points.len());
    for point in points {
        if deduped.last().copied() != Some(point) {
            deduped.push(point);
        }
    }
    deduped
}

fn distinct_point_count(points: &[(f64, f64)]) -> usize {
    let mut distinct = Vec::new();
    for point in points {
        if !distinct.contains(point) {
            distinct.push(*point);
        }
    }
    distinct.len()
}

fn line_length(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|segment| {
            let dx = segment[1].0 - segment[0].0;
            let dy = segment[1].1 - segment[0].1;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{parse_geometry_to_linestring_wkt, parse_geometry_to_wkt};
    use serde_json::json;

    #[test]
    fn polygon_geometry_is_closed_in_generic_wkt() {
        let geometry = json!({
            "type": "Polygon",
            "coordinates": [[
                [6.0, 45.0],
                [6.1, 45.0],
                [6.1, 45.1],
                [6.0, 45.1]
            ]]
        });

        assert_eq!(
            parse_geometry_to_wkt(&geometry).as_deref(),
            Some("POLYGON((6 45, 6.1 45, 6.1 45.1, 6 45.1, 6 45))")
        );
    }

    #[test]
    fn polygon_geometry_is_normalized_to_linestring() {
        let geometry = json!({
            "type": "Polygon",
            "coordinates": [[
                [6.0, 45.0],
                [6.1, 45.0],
                [6.1, 45.1],
                [6.0, 45.1]
            ]]
        });

        assert_eq!(
            parse_geometry_to_linestring_wkt(&geometry).as_deref(),
            Some("LINESTRING(6 45, 6.1 45, 6.1 45.1, 6 45.1, 6 45)")
        );
    }

    #[test]
    fn multilinestring_uses_longest_segment() {
        let geometry = json!({
            "type": "MultiLineString",
            "coordinates": [
                [[6.0, 45.0], [6.01, 45.01]],
                [[6.0, 45.0], [6.2, 45.2], [6.3, 45.3]]
            ]
        });

        assert_eq!(
            parse_geometry_to_linestring_wkt(&geometry).as_deref(),
            Some("LINESTRING(6 45, 6.2 45.2, 6.3 45.3)")
        );
    }
}
