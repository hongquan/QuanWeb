$(function() {
  var address = window.location.href;
  $('a.onbar').each(function(idx, elm) {
    var cat = elm.href;
    if (address.indexOf(cat) != -1) {
      $(elm).addClass('active');
    }
  })
})
