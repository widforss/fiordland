import {OlObjects} from "../ol";
import Feature from "ol/Feature";
import {Coordinate} from "ol/coordinate";
import Overlay from "ol/Overlay";
import GeoJSON from "ol/format/GeoJSON";

const EXPOSITIONS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

function setPopup(coordinate: Coordinate, ol: OlObjects) {
    let [overlay, content] = ol.popupOverlay;
    overlay.setPosition(coordinate);
    if (!coordinate) {
        while (content.firstChild) content.firstChild.remove();
    }
}

function preparePopup(title: string, feature: Feature, ol: OlObjects) {
    let [_, content] = ol.popupOverlay;
    content.appendChild(formatEventInfo_(title, feature));
}

function createPopupOverlay(ol: OlObjects): [Overlay, HTMLDivElement] {
    let container = document.getElementById('popup');
    let content = document.getElementById('popup-content') as HTMLDivElement;
    let closer = document.getElementById('popup-closer');

    let overlay = new Overlay({
        element: container,
        autoPan: false,
    });

    closer.onclick = function() {
        setPopup(undefined, ol);
        closer.blur();
        return false;
    };

    return [overlay, content];
}

function formatEventInfo_(header: string, event: Feature): HTMLDivElement {
    let container = document.createElement("div");

    let htmlText: string[] = [];
    let title = (text: string) => `<span class="bold">${text}</span>`;

    let date = event.get("date");
    let time = event.get("time");
    let message = event.get("message");

    if (date && time) {
        htmlText.push(`${title("Date:")} ${date} ${time}`);
    } else if (date) {
        htmlText.push(`${title("Date:")} ${date}`);
    } else if (time) {
        htmlText.push(`${title("Date:")} ${time}`);
    }
    if (message) htmlText.push(`${title("Message:")} ${message}`);

    container.innerHTML = `<h3>${header}</h3>` + htmlText.join("<br>\n") + "<br>\n";

    return container;
}

export {setPopup, preparePopup, createPopupOverlay};