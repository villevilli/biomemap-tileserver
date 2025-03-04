let map = L.map('map', {
    crs: L.CRS.Simple,
}).setView([0.0, 0.0], 0);


L.tileLayer('http://localhost:3000/biomemap/{z}/{x}/{y}.png', {
    maxNativeZoom: 0,
    minNativeZoom: -8,
    maxZoom: 17,
    minZoom: -10,
    className: 'noblur'
}).addTo(map);