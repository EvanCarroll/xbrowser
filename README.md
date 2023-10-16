`xbrowser`
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
Usage: xbrowser [OPTIONS] --user <USER> <ACTION> <BROWSER> <COMMAND>

Commands:
  cookies  Decodes Cookies
  help     Print this message or the help of the given subcommand(s)
```

For more information on `xbrowser` see `xbrowser --help` and `xbrowser export chrome cookies --help`


Here is an example

```
xbrowser export chrome cookes vpn.msauth.com
```

Installation
----

### Prerequisites

Currently building xbrowser requires the following dependencies


```sh
sudo apt install pkg-config libsecret-1-dev
```

After you have these set up, the rest is pretty easy:


```
git clone https://github.com/EvanCarroll/xbrowser.git
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
