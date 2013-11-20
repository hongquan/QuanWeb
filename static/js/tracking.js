$(document).ready(function(){

  //Configure the jQuery cookie plugin to use JSON.
  $.cookie.json = true;

  //Set the amount of time a session should last.
  var sessionExpireTime = new Date();
  sessionExpireTime.setMinutes(sessionExpireTime.getMinutes()+30);

  //Check if we have a session cookie:
  var session_cookie = $.cookie("session_cookie");

  //If it is undefined, set a new one.
  if(session_cookie == undefined){
    $.cookie("session_cookie", {
      id: Math.uuid()
    }, {
      expires: sessionExpireTime,
      path: "/" //Makes this cookie readable from all pages
    });
  }
  //If it does exist, delete it and set a new one with new expiration time
  else{
    $.removeCookie("session_cookie", {
      path: "/"
    });
    $.cookie("session_cookie", session_cookie, {
      expires: sessionExpireTime,
      path: "/"
    });
  }

  var permanent_cookie = $.cookie("permanent_cookie");

  //If it is undefined, set a new one.
  if(permanent_cookie == undefined){
    $.cookie("permanent_cookie", {
      id: Math.uuid()
    }, {
      expires: 3650, //10 year expiration date
      path: "/" //Makes this cookie readable from all pages
    });
  }

  //Add a pageview event in Keen IO
  var fullUrl = window.location.href;
  var parsedUrl = $.url(fullUrl);
  var parser = new UAParser();

  var eventProperties = {
    session_id: $.cookie("session_cookie")["id"],
    url: {
      source: parsedUrl.attr("source"),
      protocol: parsedUrl.attr("protocol"),
      domain: parsedUrl.attr("host"),
      port: parsedUrl.attr("port"),
      path: parsedUrl.attr("path"),
      query_params: parsedUrl.param(),
      anchor: parsedUrl.attr("anchor")
    },
    user_agent: {
      browser: parser.getBrowser(),
      engine: parser.getEngine(),
      os: parser.getOS()
    },
    permanent_tracker: $.cookie("permanent_cookie")["id"]
  };

  /*
  //If you know that the user is currently logged in, add information about the user.
  eventProperties["user"] = {
    id: "",
    signupDate: ""
    etc: ".."
  };
  */

  //Add information about the referrer of the same format as the current page
  var referrer = document.referrer;
  referrerObject = null;

  if(referrer != undefined){
    parsedReferrer = $.url(referrer);

    referrerObject = {
      source: parsedReferrer.attr("source"),
      protocol: parsedReferrer.attr("protocol"),
      domain: parsedReferrer.attr("host"),
      port: parsedReferrer.attr("port"),
      path: parsedReferrer.attr("path"),
      query_params: parsedReferrer.param(),
      anchor: parsedReferrer.attr("anchor")
    }
  }

  eventProperties["referrer"] = referrerObject;

  Keen.addEvent("pageviews", eventProperties)

});
