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
        "POLYGON" => parse_polygon(coords).map(|rings| {
            let rings = rings
                .into_iter()
                .map(|line| {
                    let points = line.into_iter()
                        .map(|(x, y)| format!("{} {}", x, y))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({})", points)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("POLYGON({})", rings)
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
        "MULTIPOLYGON" => parse_multi_polygon(coords).map(|polygons| {
            let polygons = polygons
                .into_iter()
                .map(|rings| {
                    let polygon_text = rings
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
                    format!("({})", polygon_text)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("MULTIPOLYGON({})", polygons)
        }),
        _ => None,
    }
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
