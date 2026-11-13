(function () {
  if (location.hostname !== "bitg.uz") return;
  if (navigator.doNotTrack === "1" || navigator.globalPrivacyControl === true) return;
  var id = 114239688;
  window.ym = window.ym || function () { (window.ym.a = window.ym.a || []).push(arguments); };
  window.ym.l = Date.now();
  var s = document.createElement("script");
  s.async = true;
  s.src = "https://mc.yandex.ru/metrika/tag.js?id=" + id;
  document.head.appendChild(s);
  window.ym(id, "init", {
    clickmap: true,
    trackLinks: true,
    accurateTrackBounce: true,
    webvisor: false,
    trackHash: false
  });
})();
