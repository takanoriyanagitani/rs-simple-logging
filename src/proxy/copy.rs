use std::collections::BTreeMap;

use crate::Item;

pub trait Proxy {
    fn get_item(&self, original: Item) -> Item;
}

struct ProxyNop {}

impl Proxy for ProxyNop {
    fn get_item(&self, original: Item) -> Item {
        original
    }
}

struct ProxyFn<P> {
    internal: P,
}

impl<P> Proxy for ProxyFn<P>
where
    P: Fn(Item) -> Item,
{
    fn get_item(&self, original: Item) -> Item {
        (self.internal)(original)
    }
}

pub fn proxy_new_from_fn<P>(internal: P) -> impl Proxy
where
    P: Fn(Item) -> Item,
{
    ProxyFn { internal }
}

pub fn proxy_join<P, Q>(p: P, q: Q) -> impl Proxy
where
    P: Proxy,
    Q: Proxy,
{
    proxy_new_from_fn(move |original: Item| {
        let after_p: Item = p.get_item(original);
        q.get_item(after_p)
    })
}

pub trait ResourceProxy {
    fn get_resource(&self, name: &str) -> Option<&str>;
}

struct ResourceProxyMap {
    internal: BTreeMap<String, String>,
}

impl ResourceProxy for ResourceProxyMap {
    fn get_resource(&self, name: &str) -> Option<&str> {
        self.internal.get(name).map(|s| s.as_str())
    }
}

pub fn resource_proxy_new_from_map(internal: BTreeMap<String, String>) -> impl ResourceProxy {
    ResourceProxyMap { internal }
}

pub fn proxy_new_from_resource_proxy<R>(resource_proxy: R) -> impl Proxy
where
    R: ResourceProxy,
{
    proxy_new_from_fn(move |mut original: Item| {
        for pair in original.resource.iter_mut() {
            let (key, val) = pair;
            let neo: Option<&str> = resource_proxy.get_resource(key);
            match neo {
                None => {}
                Some(v) => {
                    let s: String = v.into();
                    *val = s
                }
            }
        }
        original
    })
}
