import * as leaflet from "https://unpkg.com/leaflet/dist/leaflet-src.esm.js";


class MousePositionControl extends leaflet.Control {
    element: HTMLElement;

    constructor() {
        super({ position: "bottomleft" });
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


let base_layer = leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
});

let shaded_base_layer = leaflet.tileLayer('http://localhost:3000/biomemap_shaded/{z}/{x}/{y}.png', {
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
});

let contour_layer = leaflet.tileLayer('http://localhost:3000/contours/{z}/{x}/{y}.png', {
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
});

let base_maps = {
    "Normal": base_layer,
    "Shaded": shaded_base_layer,
};

let overlays = {
    "contours": contour_layer,
};

let map = leaflet.map('map', {
    crs: leaflet.CRS.Simple,
    layers: [base_layer, shaded_base_layer, contour_layer]
}).setView([0.0, 0.0], 0);

let layer_control = leaflet.control.layers(base_maps, overlays).addTo(map);

let mousePosControl = new MousePositionControl;
map.addControl(mousePosControl);

map.on("mousemove", (e) => {
    let zoom = map.getZoom();
    mousePosControl.update(e.latlng, zoom);
});

