import * as leaflet from "https://unpkg.com/leaflet/dist/leaflet-src.esm.js";

let map = leaflet.map('map', {
    crs: leaflet.CRS.Simple,
}).setView([0.0, 0.0], 0);

class MousePositionControl extends leaflet.Control {
    element: HTMLElement;

    constructor() {
        super({ position: "bottomleft" })

    }

    onAdd(map: leaflet.Map): HTMLElement {
        var latlng = leaflet.DomUtil.create("div", 'mouseposition leaflet-control-attribution');
        this.element = latlng;
        return latlng;
    }

    update(latlng: leaflet.LatLng, zoom: number) {
        this.element.innerHTML = `x: ${Math.round(latlng.lng)} z: ${Math.round(latlng.lat)} zoom: ${zoom}`;
    }
}

let mousePosControl = new MousePositionControl;

map.addControl(mousePosControl);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
    className: 'noblur'
}).addTo(map);

map.on("mousemove", (e) => {
    let zoom = map.getZoom();
    mousePosControl.update(e.latlng, zoom);
})