#!/usr/bin/env python3
"""
Web Documentation Resolver - Resolve queries or URLs into compact, LLM-ready markdown.

A lightweight wrapper that uses available web tools in cascade:
1. Check for llms.txt first
2. Use webfetch for URL extraction
3. Use websearch for query resolution

For full Python-based cascade (Exa, Tavily, Firecrawl), see:
https://github.com/d-oit/web-doc-resolver
"""

import argparse
import json
import logging
import os
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Any

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

MAX_CHARS = int(os.getenv("WEB_RESOLVER_MAX_CHARS", "8000"))
MIN_CHARS = int(os.getenv("WEB_RESOLVER_MIN_CHARS", "200"))


@dataclass
class ResolvedResult:
    """Result of a resolution."""
    source: str
    content: str
    url: str | None = None
    query: str | None = None
    error: str | None = None
    validated_links: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "source": self.source,
            "content": self.content,
            "url": self.url,
            "query": self.query,
            "error": self.error,
            "validated_links": self.validated_links,
        }


def is_url(input_str: str) -> bool:
    """Check if input is a URL."""
    if not input_str or not input_str.strip():
        return False
    return input_str.lower().startswith(("http://", "https://"))


def check_llms_txt(url: str) -> str | None:
    """Check if llms.txt exists at the site."""
    try:
        parsed_url = url if url.startswith("http") else f"https://{url}"
        llms_url = parsed_url.rstrip("/") + "/llms.txt"
        
        result = subprocess.run(
            ["webfetch", "--format", "text", llms_url],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0 and result.stdout:
            content = result.stdout.strip()
            if len(content) >= MIN_CHARS:
                logger.info(f"Found llms.txt at {llms_url}")
                return content
    except Exception as e:
        logger.debug(f"llms.txt check failed: {e}")
    return None


def fetch_with_webfetch(url: str) -> ResolvedResult | None:
    """Fetch URL content using webfetch tool."""
    try:
        result = subprocess.run(
            ["webfetch", "--format", "markdown", url],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        if result.returncode == 0 and result.stdout:
            content = result.stdout.strip()
            if len(content) >= MIN_CHARS:
                return ResolvedResult(
                    source="webfetch",
                    content=content[:MAX_CHARS],
                    url=url
                )
    except Exception as e:
        logger.debug(f"webfetch failed: {e}")
    return None


def search_with_websearch(query: str, num_results: int = 5) -> ResolvedResult | None:
    """Search using websearch tool."""
    try:
        result = subprocess.run(
            ["websearch", "--num-results", str(num_results), query],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        if result.returncode == 0 and result.stdout:
            content = result.stdout.strip()
            if len(content) >= MIN_CHARS:
                return ResolvedResult(
                    source="websearch",
                    content=content[:MAX_CHARS],
                    query=query
                )
    except Exception as e:
        logger.debug(f"websearch failed: {e}")
    return None


def resolve_url(url: str) -> dict[str, Any]:
    """Resolve a URL using the cascade."""
    logger.info(f"Resolving URL: {url}")
    
    # Step 1: Check for llms.txt
    llms_content = check_llms_txt(url)
    if llms_content:
        return ResolvedResult(
            source="llms.txt",
            content=llms_content[:MAX_CHARS],
            url=url
        ).to_dict()
    
    # Step 2: Use webfetch
    webfetch_result = fetch_with_webfetch(url)
    if webfetch_result:
        return webfetch_result.to_dict()
    
    # Step 3: Fall back to websearch
    search_result = search_with_websearch(url)
    if search_result:
        return search_result.to_dict()
    
    # All methods failed
    return ResolvedResult(
        source="none",
        content=f"# Unable to resolve URL: {url}\n\nAll resolution methods failed.",
        url=url,
        error="No resolution method available"
    ).to_dict()


def resolve_query(query: str) -> dict[str, Any]:
    """Resolve a query using the cascade."""
    logger.info(f"Resolving query: {query}")
    
    # Step 1: Use websearch
    search_result = search_with_websearch(query)
    if search_result:
        return search_result.to_dict()
    
    # All methods failed
    return ResolvedResult(
        source="none",
        content=f"# Unable to resolve query: {query}\n\nSearch failed.",
        query=query,
        error="No resolution method available"
    ).to_dict()


def resolve(input_str: str) -> dict[str, Any]:
    """Main entry point - resolve URL or query."""
    if is_url(input_str):
        return resolve_url(input_str)
    else:
        return resolve_query(input_str)


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Resolve queries or URLs into compact, LLM-ready markdown"
    )
    parser.add_argument("input", help="URL or search query to resolve")
    parser.add_argument("--max-chars", type=int, default=MAX_CHARS, help="Max characters in output")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--log-level", default="INFO", help="Log level")
    
    args = parser.parse_args()
    
    logging.getLogger().setLevel(getattr(logging, args.log_level.upper()))
    
    result = resolve(args.input)
    
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(result["content"])


if __name__ == "__main__":
    main()
