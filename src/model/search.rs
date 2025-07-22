pub trait SearchEngine: Send + Sync {
    fn generate_url(&self, query: &str) -> String;
    fn name(&self) -> String;
}

#[derive(Clone, Debug, Default)]
pub struct Duckduckgo {}

impl SearchEngine for Duckduckgo {
    fn generate_url(&self, query: &str) -> String {
        format!("https://duckduckgo.com/?q={query}")
    }

    fn name(&self) -> String {
        "duckduckgo".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DuckduckgoLite {}

impl SearchEngine for DuckduckgoLite {
    fn generate_url(&self, query: &str) -> String {
        format!("https://lite.duckduckgo.com/lite?q={query}")
    }

    fn name(&self) -> String {
        "duckduckgo".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DuckduckgoHtml {}

impl SearchEngine for DuckduckgoHtml {
    fn generate_url(&self, query: &str) -> String {
        format!("https://html.duckduckgo.com/html?q={query}")
    }

    fn name(&self) -> String {
        "duckduckgo".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Google {}

impl SearchEngine for Google {
    fn generate_url(&self, query: &str) -> String {
        format!("https://www.google.com/search?q={query}")
    }

    fn name(&self) -> String {
        "google".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bing {}

impl SearchEngine for Bing {
    fn generate_url(&self, query: &str) -> String {
        format!("https://www.bing.com/search?q={query}")
    }

    fn name(&self) -> String {
        "bing".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Baidu {}

impl SearchEngine for Baidu {
    fn generate_url(&self, query: &str) -> String {
        format!("https://www.baidu.com/s?wd={query}")
    }

    fn name(&self) -> String {
        "baidu".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sogou {}

impl SearchEngine for Sogou {
    fn generate_url(&self, query: &str) -> String {
        format!("https://www.sogou.com/web?query={query}")
    }

    fn name(&self) -> String {
        "sogou".to_string()
    }
}
