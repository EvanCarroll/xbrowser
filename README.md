`browser_cookie`
====

Read your browsers cookies from the command line! Allow easy session jacking
and integration between the browser and headless tooling. This is useful when
authentication is done online and requires JavaScript.

Currently we have the ability to 

* Decode v10 Chrome Cookies as used in Linux
* Dump Firefox cookies in Linux

This will be expanded to support all browsers, on Windows, Linux, and OSX.

Synopsis
--------

```shell
browser_cookie --os linux --browser chrome --domain vpn.msauth.com
```

Installation
----


```
git clone https://github.com/EvanCarroll/browser_cookie.git
cd browser_cookie;
cargo install --path .
```

Similar Works
----

* [Hack Browser Data](https://github.com/moonD4rk/HackBrowserData.git) a purely exfiltration tool written in Go.
* [PyCookieCheat](https://github.com/n8henrie/pycookiecheat) Python full featured cookie dump
* [Browser Cookie](https://github.com/richardpenman/browsercookie) **UNMAINTAINED** the first MVP in Python, forked to [Browser Cookie 3](https://github.com/borisbabic/browser_cookie3) for modern development.

----

This is a seed project. Currently it's pre-alpha. GitHub Stars welcome.
Progress tracked on Issue Board. All welcome to file.
