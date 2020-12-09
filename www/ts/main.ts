import * as Ol from "./ol";
import Feature from "ol/Feature";
import * as Popup from "./ol/popup";
let phone = new URL(window.location.href).searchParams.get("map");
let ws: WebSocket;
if (!/^\d{5,30}$/.test(phone)) {
    window.location.replace("/");
}

document.addEventListener('DOMContentLoaded', () => {
    let ol = Ol.initMap();
    wsHandler(ol)

    ol.map.on('singleclick', (event) => {
        let handled = false;
        ol.map.forEachFeatureAtPixel(event.pixel, (feature: Feature, layer) => {
            if (layer == ol.traceLayer) {
                Popup.preparePopup('Checkin', feature, ol);
            } else if (layer == ol.routeLayer) {
                Popup.preparePopup('Planning', feature, ol);
            }

            if (layer == ol.traceLayer || layer == ol.routeLayer) {
                Popup.setPopup(event.coordinate, ol);
                handled = true;
            }
        });
        if (!handled) {
            Popup.setPopup(undefined, ol);
        }
    });
});

function wsHandler(ol: Ol.OlObjects) {
    ws = new WebSocket("wss://fiordland.antarkt.is/listen");

    ws.onopen = (event) => {
        ws.send("+" + phone);
    }
    ws.onerror = () => {
        window.location.replace("/");
    }
    ws.onclose = ws.onerror

    ws.onmessage = (event) => {
        let json = JSON.parse(event.data);
        if (!json) {
            window.location.replace("/");
            return
        }
        Ol.loadGeoJson(json["route"], ol.routeLayer);
        Ol.loadGeoJson(json["trace"], ol.traceLayer);
    }
}