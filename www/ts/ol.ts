import * as Layer from "./ol/layer";
import * as Popup from "./ol/popup";
import TileLayer from 'ol/layer/Tile';
import Overlay from 'ol/Overlay';
import Map from 'ol/Map';
import VectorLayer from "ol/layer/Vector";
import GeoJSON, { GeoJSONFeatureCollection } from "ol/format/GeoJSON";
import Vector from "ol/source/Vector";
import VectorImageLayer from "ol/layer/Vector";

interface OlObjects {
    map: Map,
    baseLayerNo: TileLayer,
    baseLayerSe: TileLayer,
    routeLayer: VectorLayer,
    traceLayer: VectorLayer,
    backoff_counter_no: Record<string, number>,
    backoff_counter_se: Record<string, number>,
    popupOverlay: [Overlay, HTMLDivElement],
}

function initMap(): OlObjects {
    let backoff_counter_no: Record<string, number> = {};
    let backoff_counter_se: Record<string, number> = {};
    let baseLayerNo = Layer.createBaseLayerNo(backoff_counter_no);
    let baseLayerSe = Layer.createBaseLayerSe(backoff_counter_se);
    let routeLayer = Layer.createVectorLayer('#36e', 2);
    let traceLayer = Layer.createVectorLayer('#e63', 3);

    let ol: OlObjects = {
        map: null,
        baseLayerNo,
        baseLayerSe,
        routeLayer,
        traceLayer,
        backoff_counter_no,
        backoff_counter_se,
        popupOverlay: null,
    };
    let layers = [
        baseLayerSe,
        baseLayerNo,
        routeLayer,
        traceLayer,
    ];
    let [overlay, content] = Popup.createPopupOverlay(ol);
    ol.map = Layer.createMap(layers, [overlay]);
    ol.popupOverlay = [overlay, content];
    return ol;
}

function loadGeoJson(json: GeoJSONFeatureCollection[], layer: VectorImageLayer) {
    layer.getSource().clear();
    if (!json) return;
    var vectorSource = new Vector({
        features: new GeoJSON().readFeatures(json),
    });
    layer.setSource(vectorSource);
}

export {
    initMap,
    loadGeoJson,
    OlObjects,
};