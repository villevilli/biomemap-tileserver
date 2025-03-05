import * as leaflet from "https://unpkg.com/leaflet/dist/leaflet-src.esm.js";

let map = leaflet.map('map', {
    crs: leaflet.CRS.Simple,
}).setView([0.0, 0.0], 0);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: 0,
    minNativeZoom: 0,
    maxZoom: 17,
    minZoom: -1,
}).addTo(map);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: -2,
    minNativeZoom: -2,
    maxZoom: -2,
    minZoom: -3,
}).addTo(map);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: -4,
    minNativeZoom: -4,
    maxZoom: -4,
    minZoom: -5,
}).addTo(map);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: -6,
    minNativeZoom: -6,
    maxZoom: -6,
    minZoom: -7,
}).addTo(map);

leaflet.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: -8,
    minNativeZoom: -8,
    maxZoom: -8,
    minZoom: -10,
}).addTo(map);