# Http Proxy IPv6 Pool

Make every request from a separate IPv6 address.

https://zu1k.com/posts/tutorials/http-proxy-ipv6-pool/

## Tutorial

Assuming you already have an entire IPv6 subnet routed to your server, for me I purchased [Vultr's server](https://www.vultr.com/?ref=9039594-8H) to get one.

Get your IPv6 subnet prefix, for me is `2001:19f0:6001:48e4::/64`.

```sh
$ ip a
......
2: enp1s0: <BROADCAST,MULTICAST,ALLMULTI,UP,LOWER_UP> mtu 1500 qdisc fq state UP group default qlen 1000
    ......
    inet6 2001:19f0:6001:48e4:5400:3ff:fefa:a71d/64 scope global dynamic mngtmpaddr 
       valid_lft 2591171sec preferred_lft 603971sec
    ......
```

Add route via default internet interface

```sh
ip route add local 2001:19f0:6001:48e4::/64 dev enp1s0
```

Open `ip_nonlocal_bind` for binding any IP address:

```sh
sysctl net.ipv6.ip_nonlocal_bind=1
```

For IPv6 NDP, install `ndppd`:

```sh
apt install ndppd
```

then edit `/etc/ndppd.conf`:


```conf
route-ttl 30000

proxy eth0 {
    router no
    timeout 500
    ttl 30000

    rule 2001:19f0:6001:48e4::/64 {
        static
    }
}
```

Now you can test by using `curl`:

```sh
$ curl --interface 2001:19f0:6001:48e4::1 ipv6.ip.sb
2001:19f0:6001:48e4::1

$ curl --interface 2001:19f0:6001:48e4::2 ipv6.ip.sb
2001:19f0:6001:48e4::2
```

Great!

Finally, use the http proxy provided by this project:

```sh
$ while true; do curl -x http://127.0.0.1:51080 ipv6.ip.sb; done
2001:19f0:6001:48e4:971e:f12c:e2e7:d92a
2001:19f0:6001:48e4:6d1c:90fe:ee79:1123
2001:19f0:6001:48e4:f7b9:b506:99d7:1be9
2001:19f0:6001:48e4:a06a:393b:e82f:bffc
2001:19f0:6001:48e4:245f:8272:2dfb:72ce
2001:19f0:6001:48e4:df9e:422c:f804:94f7
2001:19f0:6001:48e4:dd48:6ba2:ff76:f1af
2001:19f0:6001:48e4:1306:4a84:570c:f829
2001:19f0:6001:48e4:6f3:4eb:c958:ddfa
2001:19f0:6001:48e4:aa26:3bf9:6598:9e82
2001:19f0:6001:48e4:be6b:6a62:f8f7:a14d
2001:19f0:6001:48e4:b598:409d:b946:17c
```

## Author

**Http Proxy IPv6 Pool** © [zu1k](https://github.com/zu1k), Released under the [MIT](./LICENSE) License.<br>

> Blog [zu1k.com](https://zu1k.com) · GitHub [@zu1k](https://github.com/zu1k) · Twitter [@zu1k_lv](https://twitter.com/zu1k_lv) · Telegram Channel [@peekfun](https://t.me/peekfun)
