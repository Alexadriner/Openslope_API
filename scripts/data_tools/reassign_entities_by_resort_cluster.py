import argparse
import logging
import math
import os
from collections import defaultdict

import requests


API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080")
API_KEY = os.getenv("API_KEY", "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA")
HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}",
}

logger = logging.getLogger("reassign_entities_by_resort_cluster")


def api_get(path):
    url = f"{API_BASE_URL}{path}"
    r = requests.get(url, params={"api_key": API_KEY}, headers=HEADERS, timeout=60)
    r.raise_for_status()
    return r.json()


def api_put(path, payload):
    url = f"{API_BASE_URL}{path}"
    r = requests.put(url, json=payload, headers=HEADERS, timeout=60)
    if r.status_code not in (200, 201):
        raise RuntimeError(f"PUT {path} failed: {r.status_code} {r.text}")


def api_delete(path):
    url = f"{API_BASE_URL}{path}"
    r = requests.delete(url, headers=HEADERS, timeout=60)
    if r.status_code not in (200, 204):
        raise RuntimeError(f"DELETE {path} failed: {r.status_code} {r.text}")


def to_float(v):
    try:
        if v is None:
            return None
        return float(v)
    except (TypeError, ValueError):
        return None


def haversine_m(a_lat, a_lon, b_lat, b_lon):
    r = 6371000.0
    p1 = math.radians(a_lat)
    p2 = math.radians(b_lat)
    dp = math.radians(b_lat - a_lat)
    dl = math.radians(b_lon - a_lon)
    x = math.sin(dp / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    return 2 * r * math.atan2(math.sqrt(x), math.sqrt(1 - x))


def resort_coord(resort):
    g = resort.get("geography") or {}
    c = g.get("coordinates") or {}
    lat = to_float(c.get("latitude", resort.get("latitude")))
    lon = to_float(c.get("longitude", resort.get("longitude")))
    if lat is None or lon is None:
        return None
    return (lat, lon)


def build_clusters_and_neighbors(resorts, cluster_distance_m):
    ids = [r["id"] for r in resorts if resort_coord(r) is not None]
    coord_map = {r["id"]: resort_coord(r) for r in resorts if resort_coord(r) is not None}
    graph = {rid: [] for rid in ids}

    for i in range(len(ids)):
        for j in range(i + 1, len(ids)):
            a = ids[i]
            b = ids[j]
            d = haversine_m(coord_map[a][0], coord_map[a][1], coord_map[b][0], coord_map[b][1])
            if d <= cluster_distance_m:
                graph[a].append(b)
                graph[b].append(a)

    neighbor_map = {rid: set(neigh) for rid, neigh in graph.items()}

    seen = set()
    clusters = []
    for rid in ids:
        if rid in seen:
            continue
        stack = [rid]
        seen.add(rid)
        comp = []
        while stack:
            cur = stack.pop()
            comp.append(cur)
            for nxt in graph[cur]:
                if nxt not in seen:
                    seen.add(nxt)
                    stack.append(nxt)
        if len(comp) > 1:
            clusters.append(comp)
    return clusters, coord_map, neighbor_map


def entity_midpoint(entity):
    g = entity.get("geometry") or {}
    s = g.get("start") or {}
    e = g.get("end") or {}
    lat1 = to_float(s.get("latitude", entity.get("lat_start")))
    lon1 = to_float(s.get("longitude", entity.get("lon_start")))
    lat2 = to_float(e.get("latitude", entity.get("lat_end")))
    lon2 = to_float(e.get("longitude", entity.get("lon_end")))

    pts = []
    if lat1 is not None and lon1 is not None:
        pts.append((lat1, lon1))
    if lat2 is not None and lon2 is not None:
        pts.append((lat2, lon2))
    if not pts:
        return None
    if len(pts) == 1:
        return pts[0]
    return ((pts[0][0] + pts[1][0]) / 2.0, (pts[0][1] + pts[1][1]) / 2.0)


def choose_nearest_resort(point, candidate_resort_ids, coord_map):
    best = None
    best_id = None
    for rid in candidate_resort_ids:
        rc = coord_map.get(rid)
        if rc is None:
            continue
        d = haversine_m(point[0], point[1], rc[0], rc[1])
        if best is None or d < best:
            best = d
            best_id = rid
    return best_id, best


def flatten_lift_payload(lift):
    display = lift.get("display") or {}
    geometry = lift.get("geometry") or {}
    start = geometry.get("start") or {}
    end = geometry.get("end") or {}
    specs = lift.get("specs") or {}
    source = lift.get("source") or {}
    status = lift.get("status") or {}
    return {
        "resort_id": lift.get("resort_id"),
        "name": lift.get("name"),
        "lift_type": display.get("lift_type") or lift.get("lift_type") or "chairlift",
        "capacity_per_hour": specs.get("capacity_per_hour"),
        "seats": specs.get("seats"),
        "bubble": bool(specs.get("bubble")),
        "heated_seats": bool(specs.get("heated_seats")),
        "year_built": specs.get("year_built"),
        "altitude_start_m": specs.get("altitude_start_m"),
        "altitude_end_m": specs.get("altitude_end_m"),
        "lat_start": start.get("latitude", lift.get("lat_start")),
        "lon_start": start.get("longitude", lift.get("lon_start")),
        "lat_end": end.get("latitude", lift.get("lat_end")),
        "lon_end": end.get("longitude", lift.get("lon_end")),
        "source_system": source.get("system") or lift.get("source_system") or "osm",
        "source_entity_id": source.get("entity_id") or lift.get("source_entity_id"),
        "name_normalized": display.get("normalized_name") or lift.get("name_normalized"),
        "operational_status": status.get("operational_status") or lift.get("operational_status") or "unknown",
        "operational_note": status.get("note") or lift.get("operational_note"),
        "planned_open_time": status.get("planned_open_time") or lift.get("planned_open_time"),
        "planned_close_time": status.get("planned_close_time") or lift.get("planned_close_time"),
        "status_updated_at": status.get("updated_at") or lift.get("status_updated_at"),
        "status_source_url": source.get("source_url") or lift.get("status_source_url"),
    }


def flatten_slope_payload(slope):
    display = slope.get("display") or {}
    geometry = slope.get("geometry") or {}
    start = geometry.get("start") or {}
    end = geometry.get("end") or {}
    specs = slope.get("specs") or {}
    source = slope.get("source") or {}
    status = slope.get("status") or {}
    return {
        "resort_id": slope.get("resort_id"),
        "name": slope.get("name"),
        "difficulty": display.get("difficulty") or slope.get("difficulty") or "blue",
        "length_m": specs.get("length_m"),
        "vertical_drop_m": specs.get("vertical_drop_m"),
        "average_gradient": specs.get("average_gradient"),
        "max_gradient": specs.get("max_gradient"),
        "snowmaking": bool(specs.get("snowmaking")),
        "night_skiing": bool(specs.get("night_skiing")),
        "family_friendly": bool(specs.get("family_friendly")),
        "race_slope": bool(specs.get("race_slope")),
        "lat_start": start.get("latitude", slope.get("lat_start")),
        "lon_start": start.get("longitude", slope.get("lon_start")),
        "lat_end": end.get("latitude", slope.get("lat_end")),
        "lon_end": end.get("longitude", slope.get("lon_end")),
        "source_system": source.get("system") or slope.get("source_system") or "osm",
        "source_entity_id": source.get("entity_id") or slope.get("source_entity_id"),
        "name_normalized": display.get("normalized_name") or slope.get("name_normalized"),
        "operational_status": status.get("operational_status") or slope.get("operational_status") or "unknown",
        "grooming_status": status.get("grooming_status") or slope.get("grooming_status") or "unknown",
        "operational_note": status.get("note") or slope.get("operational_note"),
        "status_updated_at": status.get("updated_at") or slope.get("status_updated_at"),
        "status_source_url": source.get("source_url") or slope.get("status_source_url"),
    }


def reassign_entities(entity_type, entities, neighbor_map, coord_map, switch_margin_m, apply):
    total_reassigned = 0
    for entity in entities:
        current_resort = entity.get("resort_id")
        if current_resort not in neighbor_map:
            continue

        pt = entity_midpoint(entity)
        if pt is None:
            continue

        # Only compare with direct nearby resorts to avoid transitive mega-clusters.
        candidates = set(neighbor_map.get(current_resort, set()))
        candidates.add(current_resort)
        nearest_id, nearest_dist = choose_nearest_resort(pt, candidates, coord_map)
        if not nearest_id or nearest_id == current_resort:
            continue

        cur_coord = coord_map.get(current_resort)
        if cur_coord is None:
            continue
        current_dist = haversine_m(pt[0], pt[1], cur_coord[0], cur_coord[1])

        if nearest_dist + switch_margin_m >= current_dist:
            continue

        payload = flatten_lift_payload(entity) if entity_type == "lifts" else flatten_slope_payload(entity)
        payload["resort_id"] = nearest_id

        logger.info(
            "%s #%s reassigned %s -> %s (current=%.1fm nearest=%.1fm)",
            entity_type[:-1].upper(),
            entity.get("id"),
            current_resort,
            nearest_id,
            current_dist,
            nearest_dist,
        )

        if apply:
            api_put(f"/{entity_type}/{entity['id']}", payload)
        total_reassigned += 1

    return total_reassigned


def dedup_by_source_id(entity_type, entities, apply):
    groups = defaultdict(list)
    for e in entities:
        src = (e.get("source") or {}).get("entity_id") or e.get("source_entity_id")
        resort_id = e.get("resort_id")
        if not src or not resort_id:
            continue
        groups[(resort_id, src)].append(e)

    deleted = 0
    for (_resort_id, _src), rows in groups.items():
        if len(rows) < 2:
            continue
        rows_sorted = sorted(rows, key=lambda x: int(x.get("id", 0)))
        keep = rows_sorted[0]
        for drop in rows_sorted[1:]:
            logger.info(
                "%s duplicate source_entity_id cleanup: keep #%s delete #%s (resort=%s source=%s)",
                entity_type[:-1].upper(),
                keep.get("id"),
                drop.get("id"),
                drop.get("resort_id"),
                (drop.get("source") or {}).get("entity_id") or drop.get("source_entity_id"),
            )
            if apply:
                api_delete(f"/{entity_type}/{drop['id']}")
            deleted += 1
    return deleted


def main():
    parser = argparse.ArgumentParser(
        description="Reassign lifts/slopes uniquely inside close-by resort clusters."
    )
    parser.add_argument("--cluster-km", type=float, default=9.0)
    parser.add_argument(
        "--switch-margin-m",
        type=float,
        default=250.0,
        help="Require nearest resort to be at least this much closer before reassigning.",
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO, format="%(asctime)s | %(levelname)s | %(message)s")
    apply_changes = not args.dry_run
    logger.info("Starting cluster reassignment. mode=%s", "apply" if apply_changes else "dry-run")

    resorts = api_get("/resorts")
    clusters, coord_map, neighbor_map = build_clusters_and_neighbors(resorts, args.cluster_km * 1000.0)
    logger.info("Found %s close-resort clusters (threshold %.1f km)", len(clusters), args.cluster_km)

    if not clusters:
        logger.info("No clusters found. Done.")
        return

    lifts = api_get("/lifts")
    slopes = api_get("/slopes")

    lift_reassigned = reassign_entities(
        "lifts", lifts, neighbor_map, coord_map, args.switch_margin_m, apply_changes
    )
    slope_reassigned = reassign_entities(
        "slopes", slopes, neighbor_map, coord_map, args.switch_margin_m, apply_changes
    )

    # Reload for dedup, only if applied.
    if apply_changes:
        lifts = api_get("/lifts")
        slopes = api_get("/slopes")

    lift_deleted = dedup_by_source_id("lifts", lifts, apply_changes)
    slope_deleted = dedup_by_source_id("slopes", slopes, apply_changes)

    logger.info(
        "Cluster reassignment done. reassigned lifts=%s slopes=%s | deleted duplicates lifts=%s slopes=%s",
        lift_reassigned,
        slope_reassigned,
        lift_deleted,
        slope_deleted,
    )


if __name__ == "__main__":
    main()
