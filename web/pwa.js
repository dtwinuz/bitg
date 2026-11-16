// konvertorni ilova sifatida o'rnatish (PWA, Microsoft Store MSIX) uchun service worker'ni ulaydi
'use strict';
if ('serviceWorker' in navigator && location.protocol === 'https:') {
  window.addEventListener('load', function () {
    navigator.serviceWorker.register('/sw.js', { scope: '/konvertor/' }).catch(function () {});
  });
}
