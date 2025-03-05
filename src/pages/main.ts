import * as leaflet from "https://unpkg.com/leaflet/dist/leaflet-src.esm.js";

let map = leaflet.map('map', {
    crs: leaflet.CRS.Simple,
}).setView([0.0, 0.0], 0);

let MousePositionControl = leaflet.Control.extend({
    _container: null,
    options: {
        position: 'bottomleft'
    },

    onAdd: function (map: leaflet.Map) {
        var latlng = leaflet.DomUtil.create('div', 'mouseposition leaflet-control-attribution');
        this._latlng = latlng;
        return latlng;
    },

    updateHTML: function (latlng: leaflet.LatLng) {
        this._latlng.innerHTML = `x: ${Math.round(latlng.lat)} z: ${Math.round(latlng.lng)}`;
    }
});

let control = new MousePositionControl

map.addControl(control)

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: 0,
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
    className: 'noblur'
}).addTo(map);

map.on("mousemove", (e) => {
    control.updateHTML(e.latlng)
})