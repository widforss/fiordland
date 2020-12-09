import Overlay from "ol/Overlay";
import Map from "ol/Map";
import {Coordinate} from "ol/coordinate";
import {defaults as defaultInteractions} from "ol/interaction";
import {Extent} from "ol/extent";
import View, {ViewOptions} from "ol/View";
import TileLayer from "ol/layer/Tile";
import WMTS from "ol/source/WMTS";
import WMTSTileGrid from "ol/tilegrid/WMTS";
import {TileSourceEvent} from "ol/source/Tile";
import Tile from "ol/Tile";
import {register} from "ol/proj/proj4";
import proj4 from "proj4";
import LayerGroup from "ol/layer/Group";
import {Layer} from "ol/layer";
import XYZ from 'ol/source/XYZ';
import VectorImageLayer from "ol/layer/Vector";
import Vector from "ol/source/Vector";
import Style from "ol/style/Style";
import Fill from "ol/style/Fill";
import Stroke from "ol/style/Stroke";
import CircleStyle from "ol/style/Circle";
import ImageStyle from "ol/style/Image";

const EXP_TIMEOUT = 500;
const ATTR_LM = [
    '© <a href="https://www.lantmateriet.se/" target="_blank">Lantmäteriet</a>',
    '<a href="https://www.lantmateriet.se/sv/Kartor-och-geografisk-information/oppna-data/" target="_blank">(CC0 1.0)</a>'
].join(" ");
const ATTR_KV = [
    '© <a href="https://www.kartverket.no/" target="_blank">Kartverket</a>',
    '<a href="https://www.kartverket.no/data/lisens/" target="_blank">(CC BY 4.0)</a>'
].join(" ");
const INIT_POS = [438700, 7264409];
const INIT_ZOOM = 7;
const TILE_URL_SE = 'https://api.lantmateriet.se/open/topowebb-ccby/v1/wmts/' +
                    'token/f6004f59-323f-36ac-b83c-be300ee533d7/?' +
                    'SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=topowebb&' +
                    'STYLE=default&TILEMATRIXSET=3006&' +
                    'TILEMATRIX={z}&TILEROW={y}&TILECOL={x}&FORMAT=image/png';
const TILE_URL_NO = 'https://opencache.statkart.no/gatekeeper/gk/gk.open_wmts/?';
const PROJECTION = 'EPSG:25833';
const PROJECTION_EXTENT_SE: Extent = [-1200000, 4700000, 2600000, 8500000];
const PROJECTION_EXTENT_NO: Extent = [-2500000, 6420992, 1130000, 9045984];
const MIN_ZOOM = 7;
const MAX_ZOOM = 17;
const RESOLUTIONS_SE = [
    4096,
    2048,
    1024,
    512,
    256,
    128,
    64,
    32,
    16,
    8,
    // Changed due to licensing issues.
    //4,
    //2,
    //1,
    //0.5,
];
const RESOLUTIONS_NO = [
    21664,
    10832,
    5416,
    2708,
    1354,
    677,
    338.5,
    169.25,
    84.625,
    42.3125,
    21.15625,
    10.578125,
    5.2890625,
    2.64453125,
    1.322265625,
    0.6611328125,
    0.33056640625,
    0.165283203125,
];
const MATRIX_IDS = [
    "EPSG:25833:0",
    "EPSG:25833:1",
    "EPSG:25833:2",
    "EPSG:25833:3",
    "EPSG:25833:4",
    "EPSG:25833:5",
    "EPSG:25833:6",
    "EPSG:25833:7",
    "EPSG:25833:8",
    "EPSG:25833:9",
    "EPSG:25833:10",
    "EPSG:25833:11",
    "EPSG:25833:12",
    "EPSG:25833:13",
    "EPSG:25833:14",
    "EPSG:25833:15",
    "EPSG:25833:16",
    "EPSG:25833:17",
];

proj4.defs('EPSG:25833', '+proj=utm +zone=33 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs');
register(proj4);

function createMap(layers: (Layer | LayerGroup)[], overlay: Overlay[]): Map {
    let center: Coordinate = INIT_POS;
    let zoom = INIT_ZOOM;
    let map = new Map({
        layers: layers,
        overlays: overlay,
        target: 'map',
        view: createView(center, zoom),
        interactions: defaultInteractions({
            altShiftDragRotate: false,
            pinchRotate: false,
        }),
    });
    return map;
}

function createView(center: Coordinate, zoom: number): View {
    let options: ViewOptions = {
        projection: PROJECTION,
        center,
        zoom,
        minZoom: MIN_ZOOM,
        maxZoom: MAX_ZOOM,
    };
    return new View(options);
}

function createBaseLayerSe(backoff_counter: Record<string, number>): TileLayer {
    let baseLayer = new TileLayer({
        source: new XYZ({
            url: TILE_URL_SE,
            attributions: ATTR_LM,
            tileGrid: new WMTSTileGrid({
                extent: PROJECTION_EXTENT_SE,
                resolutions: RESOLUTIONS_SE,
                matrixIds: MATRIX_IDS,
            }),
            projection: PROJECTION,
            wrapX: false,
        }),
        zIndex: 0,
    });
    baseLayer.getSource().on('tileloaderror', function (e: TileSourceEvent) {
        exponentialBackoff_(e.tile, backoff_counter);
    });
    return baseLayer;
}

function createBaseLayerNo(backoff_counter: Record<string, number>): TileLayer {
    let baseLayer = new TileLayer({
        source: new WMTS({
            url: TILE_URL_NO,
            attributions: ATTR_KV,
            tileGrid: new WMTSTileGrid({
                extent: PROJECTION_EXTENT_NO,
                resolutions: RESOLUTIONS_NO,
                matrixIds: MATRIX_IDS,
            }),
            layer: 'topo4',
            matrixSet: 'EPSG:25833',
            format: 'image/png',
            projection: PROJECTION,
            style: 'default',
            wrapX: false,
        }),
        zIndex: 1,
    });
    baseLayer.getSource().on('tileloaderror', function (e: TileSourceEvent) {
        exponentialBackoff_(e.tile, backoff_counter);
    });
    return baseLayer;
}

function exponentialBackoff_(tile: Tile, backoff_counter: Record<string, number>): void {
    let idx = tile.getTileCoord().toString();
    if (!(idx in backoff_counter)) {
        backoff_counter[idx] = 0;
    } else if (backoff_counter[idx] == 5) {
        return;
    }
    let delay = Math.random() * EXP_TIMEOUT * Math.pow(2, backoff_counter[idx]++);
    setTimeout(() => {
        tile.load();
    }, delay);
}

function createVectorLayer(color: string, zIndex: number): VectorImageLayer {
    return new VectorImageLayer({
        source: new Vector({
            wrapX: false,
        }),
        style: new Style({
            image: new CircleStyle({
                radius: 10,
                stroke: new Stroke({
                    color,
                    width: 3,
                }),
                fill: new Fill({
                    color: [180, 180, 180, 0.5],
                }),
            }) as ImageStyle,
        }),
        zIndex,
    });
}


export {
    createMap,
    createView,
    createBaseLayerNo,
    createBaseLayerSe,
    createVectorLayer,
};