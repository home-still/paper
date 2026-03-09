use clap::{Args, Parser, Subcommand, ValueEnum};

/// paper — meta-search tool for academic papers
///
/// Examples:
///   paper search "transformer attention mechanisms"
///   paper search --type title "neural networks" -n 5
///   paper get --doi "10.48550/arXiv.2301.00001"
///   paper search "deep learning" --output json
///   paper config show
#[derive(Parser, Debug)]
#[command(name = "paper", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: NounCmd,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    #[command(flatten)]
    pub global: hs_style::global_args::GlobalArgs,
}

impl GlobalOpts {
    pub fn quiet(&self) -> bool {
        self.global.quiet
    }

    pub fn is_json(&self) -> bool {
        matches!(
            self.global.output,
            hs_style::global_args::OutputFormat::Json
        )
    }

    pub fn color_str(&self) -> &str {
        match self.global.color {
            hs_style::global_args::ColorChoice::Auto => "auto",
            hs_style::global_args::ColorChoice::Always => "always",
            hs_style::global_args::ColorChoice::Never => "never",
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum NounCmd {
    /// Search for papers across providers
    ///
    /// Examples:
    ///   paper search "transformer attention"
    ///   paper search --type author "Hinton" -n 5
    Search {
        /// Search query string
        query: String,

        /// Type of search to perform
        #[arg(short = 't', long = "type", default_value = "keywords")]
        search_type: SearchTypeArg,

        /// Show abstracts in search results
        #[arg(short = 'a', long = "abstract")]
        show_abstract: bool,

        /// Filter by date range (e.g., ">=2025", ">2023 <2025", ">=2024-06")
        #[arg(short = 'd', long = "date")]
        date: Option<String>,

        /// Maximum number of results (1-100)
        #[arg(short = 'n', long, default_value = "10", value_parser = clap::value_parser!(u16).range(1..=100))]
        max_results: u16,

        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: usize,

        /// Provider to search
        #[arg(short, long, default_value = "arxiv")]
        provider: ProviderArg,
    },
    /// Get a single paper by DOI
    ///
    /// Examples:
    ///   paper get --doi "10.48550/arXiv.2301.00001"
    Get {
        /// DOI to look up
        #[arg(long)]
        doi: String,

        /// Provider to query
        #[arg(short, long, default_value = "arxiv")]
        provider: ProviderArg,
    },
    /// Download papers (search + download, or single DOI)
    ///
    /// Examples:
    ///   paper download "neural nets" -n 25
    ///   paper download --doi "10.48550/arXiv.2301.00001"
    Download {
        /// Search query (downloads matching papers)
        query: Option<String>,

        /// Filter by date range (e.g., ">=2025", ">2023 <2025", ">=2024-06")
        #[arg(short = 'd', long = "date")]
        date: Option<String>,

        /// Download a single paper by DOI
        #[arg(long, conflicts_with = "query")]
        doi: Option<String>,

        /// Maximum number of papers to download (1-100)
        #[arg(short = 'n', long, default_value = "10", value_parser = clap::value_parser!(u16).range(1..=100))]
        max_results: u16,

        /// Maximum concurrent downloads
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,

        /// Search type for query-based download
        #[arg(short = 't', long = "type", default_value = "keywords")]
        search_type: SearchTypeArg,

        /// Provider to search
        #[arg(short, long, default_value = "arxiv")]
        provider: ProviderArg,
    },
    /// View and manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the resolved configuration
    Show,
    /// Print the config file path
    Path,
}

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "lowercase")]
pub enum SearchTypeArg {
    Keywords,
    Title,
    Author,
    Doi,
    Subject,
}

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "lowercase")]
pub enum ProviderArg {
    Arxiv,
}

impl From<SearchTypeArg> for paper_core::models::SearchType {
    fn from(arg: SearchTypeArg) -> Self {
        match arg {
            SearchTypeArg::Keywords => Self::Keywords,
            SearchTypeArg::Title => Self::Title,
            SearchTypeArg::Author => Self::Author,
            SearchTypeArg::Doi => Self::DOI,
            SearchTypeArg::Subject => Self::Subject,
        }
    }
}
