//! Alternative items getter using non-zero copy.

use std::collections::BTreeMap;

use crate::Item;

/// Proxy can be used to get a mutated item.
pub trait Proxy {
    /// Gets a mutated item.
    fn get_item(&self, original: Item) -> Item;
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

/// Creates a proxy which uses a closure to get a mutated item.
pub fn proxy_new_from_fn<P>(internal: P) -> impl Proxy
where
    P: Fn(Item) -> Item,
{
    ProxyFn { internal }
}

/// Creates a joined proxy.
///
/// # Arguments
/// - p: Gets a mutated item from the original item.
/// - q: Gets a mutated item from the item got by `p`.
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

/// ResourceProxy can be used to get an alternative value for a key.
pub trait ResourceProxy {
    /// Tries to get a value for a name.
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

/// Creates a resource proxy from a map.
pub fn resource_proxy_new_from_map(internal: BTreeMap<String, String>) -> impl ResourceProxy {
    ResourceProxyMap { internal }
}

/// Creates a proxy which tries to get a resource value from a resource proxy.
///
/// # Arguments
/// - resource_proxy: Tries to get a resource value if exists.
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
