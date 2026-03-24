use crate::types::TechCategory;

/// Map a Wappalyzer numeric category ID to our `TechCategory` enum.
/// Returns `None` for categories we don't explicitly model (engine will
/// use the raw category name string as a fallback).
pub fn map_category(cat_id: u32) -> Option<TechCategory> {
    match cat_id {
        1 => Some(TechCategory::ContentManagementSystem),  // CMS
        2 => Some(TechCategory::Widget),                   // Message boards
        3 => Some(TechCategory::Widget),                   // Database managers
        4 => Some(TechCategory::Widget),                   // Documentation
        5 => Some(TechCategory::Widget),                   // Widgets
        6 => Some(TechCategory::ECommerce),                // Ecommerce
        7 => Some(TechCategory::Widget),                   // Photo galleries
        8 => Some(TechCategory::Widget),                   // Wikis
        9 => Some(TechCategory::Widget),                   // Hosting panels
        10 => Some(TechCategory::TrafficAnalysisTool),     // Analytics
        11 => Some(TechCategory::ContentManagementSystem), // Blogs
        12 => Some(TechCategory::JavaScriptFramework),     // JavaScript frameworks
        13 => Some(TechCategory::Widget),                  // Issue trackers
        14 => Some(TechCategory::VideoPlayer),             // Video players
        15 => Some(TechCategory::SocialWidget),            // Comment systems
        16 => Some(TechCategory::Security),                // Security
        17 => Some(TechCategory::FontScript),              // Font scripts
        18 => Some(TechCategory::JavaScriptFramework),     // Web frameworks
        19 => Some(TechCategory::Widget),                  // Miscellaneous
        20 => Some(TechCategory::Widget),                  // Editors
        21 => Some(TechCategory::Widget),                  // LMS
        22 => Some(TechCategory::WebServer),               // Web servers
        23 => Some(TechCategory::ReverseProxyService),     // Caching
        24 => Some(TechCategory::Widget),                  // Rich text editors
        25 => Some(TechCategory::JavaScriptLibrary),       // JavaScript graphics
        26 => Some(TechCategory::JavaScriptFramework),     // Mobile frameworks
        27 => Some(TechCategory::ServerSideProgrammingLanguage), // Programming languages
        28 => Some(TechCategory::Widget),                  // Operating systems
        29 => Some(TechCategory::Widget),                  // Search engines
        30 => Some(TechCategory::Widget),                  // Webmail
        31 => Some(TechCategory::ContentDeliveryNetwork),  // CDN
        32 => Some(TechCategory::TrafficAnalysisTool),     // Marketing automation
        33 => Some(TechCategory::ServerSideProgrammingLanguage), // Web server extensions
        34 => Some(TechCategory::ServerSideProgrammingLanguage), // Databases
        35 => Some(TechCategory::Map),                     // Maps
        36 => Some(TechCategory::AdvertisingNetwork),      // Advertising
        37 => Some(TechCategory::Widget),                  // Network devices
        38 => Some(TechCategory::VideoPlayer),             // Media servers
        39 => Some(TechCategory::Widget),                  // Webcams
        41 => Some(TechCategory::Payment),                 // Payment processors
        42 => Some(TechCategory::TagManager),              // Tag managers
        44 => Some(TechCategory::Widget),                  // CI
        45 => Some(TechCategory::Widget),                  // Control systems
        46 => Some(TechCategory::Widget),                  // Remote access
        47 => Some(TechCategory::Widget),                  // Development
        48 => Some(TechCategory::Widget),                  // Network storage
        49 => Some(TechCategory::Widget),                  // Feed readers
        50 => Some(TechCategory::ContentManagementSystem), // DMS
        51 => Some(TechCategory::ContentManagementSystem), // Page builders
        52 => Some(TechCategory::Widget),                  // Live chat
        53 => Some(TechCategory::Widget),                  // CRM
        54 => Some(TechCategory::Widget),                  // SEO
        55 => Some(TechCategory::Widget),                  // Accounting
        56 => Some(TechCategory::Widget),                  // Cryptominers
        57 => Some(TechCategory::ContentManagementSystem), // Static site generators
        58 => Some(TechCategory::Widget),                  // User Onboarding
        59 => Some(TechCategory::JavaScriptLibrary),       // JavaScript libraries
        60 => Some(TechCategory::Widget),                  // Containers
        61 => Some(TechCategory::Widget),                  // SaaS
        62 => Some(TechCategory::PaaS),                    // PaaS
        63 => Some(TechCategory::DataCenterProvider),      // IaaS
        64 => Some(TechCategory::ReverseProxyService),     // Reverse proxies
        65 => Some(TechCategory::ReverseProxyService),     // Load balancers
        66 => Some(TechCategory::CssFramework),            // UI frameworks
        67 => Some(TechCategory::Widget),                  // Cookie compliance
        68 => Some(TechCategory::Widget),                  // Accessibility
        69 => Some(TechCategory::Widget),                  // Social login
        70 => Some(TechCategory::SslCertificateAuthority), // SSL/TLS certificate authority
        _ => None,
    }
}

/// Category IDs that represent server-side languages/databases and should
/// also imply `ServerSideProgrammingLanguage`
pub fn is_server_side_lang(cat_id: u32) -> bool {
    matches!(cat_id, 27 | 33 | 34)
}

/// Category IDs that represent client-side scripting
pub fn is_client_side_lang(cat_id: u32) -> bool {
    matches!(cat_id, 12 | 18 | 25 | 26 | 59)
}
