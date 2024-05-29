use crate::{AppError, ProjectRoutes};
use anyhow::Result;
use arc_swap::ArcSwap;
use axum::http::Method;
use matchit::{Match, Router};
use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct SwappableAppRouter {
    pub inner: Arc<ArcSwap<AppRouterInner>>,
}

pub struct AppRouterInner {
    pub code: String,
    pub router: Router<MethodRoute>,
}

#[derive(Clone)]
pub struct AppRouter(Arc<AppRouterInner>);

#[derive(Debug, Default, Clone)]
pub struct MethodRoute {
    get: Option<String>, // handler name in js code
    head: Option<String>,
    delete: Option<String>,
    options: Option<String>,
    patch: Option<String>,
    post: Option<String>,
    put: Option<String>,
    trace: Option<String>,
    connect: Option<String>,
}

impl SwappableAppRouter {
    pub fn try_new(code: impl Into<String>, routes: ProjectRoutes) -> Result<Self> {
        let router = Self::get_router(routes)?;
        let inner = AppRouterInner::new(code, router);
        Ok(Self {
            inner: Arc::new(ArcSwap::from_pointee(inner)),
        })
    }

    pub fn swap(&self, code: impl Into<String>, routes: ProjectRoutes) -> Result<()> {
        let router = Self::get_router(routes)?;
        let inner = AppRouterInner::new(code, router);
        self.inner.store(Arc::new(inner));
        Ok(())
    }

    pub fn load(&self) -> AppRouter {
        AppRouter(self.inner.load_full())
    }

    fn get_router(routes: ProjectRoutes) -> Result<Router<MethodRoute>> {
        let mut router = Router::new();
        for (path, methods) in routes {
            let mut method_route = MethodRoute::default();
            for method in methods {
                match method.method {
                    Method::GET => method_route.get = Some(method.handler),
                    Method::HEAD => method_route.head = Some(method.handler),
                    Method::DELETE => method_route.delete = Some(method.handler),
                    Method::OPTIONS => method_route.options = Some(method.handler),
                    Method::PATCH => method_route.patch = Some(method.handler),
                    Method::POST => method_route.post = Some(method.handler),
                    Method::PUT => method_route.put = Some(method.handler),
                    Method::TRACE => method_route.trace = Some(method.handler),
                    Method::CONNECT => method_route.connect = Some(method.handler),
                    v => unreachable!("unsupported method {v}"),
                }
            }
            router.insert(path, method_route)?;
        }
        Ok(router)
    }
}

impl AppRouter {
    pub fn match_it<'m, 'p>(
        &'m self,
        method: Method,
        path: &'p str,
    ) -> Result<Match<&str>, AppError>
    where
        'p: 'm,
    {
        let Ok(ret) = self.router.at(path) else {
            return Err(AppError::RoutePathNotFound(path.to_string()));
        };

        let s = match method {
            Method::GET => ret.value.get.as_deref(),
            Method::HEAD => ret.value.head.as_deref(),
            Method::DELETE => ret.value.delete.as_deref(),
            Method::OPTIONS => ret.value.options.as_deref(),
            Method::PATCH => ret.value.patch.as_deref(),
            Method::POST => ret.value.post.as_deref(),
            Method::PUT => ret.value.put.as_deref(),
            Method::TRACE => ret.value.trace.as_deref(),
            Method::CONNECT => ret.value.connect.as_deref(),
            _ => unreachable!(),
        }
        .ok_or_else(|| AppError::RouteMethodNotAllowed(method))?;
        Ok(Match {
            value: s,
            params: ret.params,
        })
    }
}

impl Deref for AppRouter {
    type Target = AppRouterInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppRouterInner {
    pub fn new(code: impl Into<String>, router: Router<MethodRoute>) -> Self {
        Self {
            code: code.into(),
            router,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectConfig;

    #[test]
    fn app_router_match_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let router = SwappableAppRouter::try_new("", config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(m.value, "hello");
        assert_eq!(m.params.get("id"), Some("1"));

        let m = app_router.match_it(Method::POST, "/api/goodbye/2").unwrap();
        assert_eq!(m.value, "hello");
        assert_eq!(m.params.get("id"), Some("2"));
        assert_eq!(m.params.get("name"), Some("goodbye"));
    }

    #[test]
    fn app_router_swap_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let router = SwappableAppRouter::try_new("", config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(m.value, "hello");

        let new_config = include_str!("../fixtures/config1.yml");
        let new_config: ProjectConfig = serde_yaml::from_str(new_config).unwrap();
        router.swap("", new_config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(m.value, "hello1");

        let m = app_router.match_it(Method::POST, "/api/goodbye/2").unwrap();
        assert_eq!(m.value, "handler2");
    }
}
