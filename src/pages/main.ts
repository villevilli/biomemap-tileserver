import * as leaflet from "https://unpkg.com/leaflet/dist/leaflet-src.esm.js";

let map = leaflet.map('map', {
    crs: leaflet.CRS.Simple,
}).setView([0.0, 0.0], 0);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: 0,
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
    className: 'noblur'
}).addTo(map);