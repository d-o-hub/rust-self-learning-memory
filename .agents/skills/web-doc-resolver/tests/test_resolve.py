"""Tests for web-doc-resolver scripts."""

import os
import sys
from unittest.mock import Mock, patch

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from scripts.resolve import (
    MAX_CHARS,
    MIN_CHARS,
    ResolvedResult,
    check_llms_txt,
    fetch_with_webfetch,
    is_url,
    resolve,
    resolve_query,
    resolve_url,
    search_with_websearch,
)


class TestIsUrl:
    """Test URL detection."""

    def test_valid_urls(self):
        """Test that valid URLs are detected."""
        assert is_url("https://example.com")
        assert is_url("http://example.com/path")
        assert is_url("https://docs.rs/tokio")

    def test_invalid_urls(self):
        """Test that non-URLs are rejected."""
        assert not is_url("not a url")
        assert not is_url("just some text")
        assert not is_url("")
        assert not is_url("machine learning")

    def test_query_with_special_characters(self):
        """Test queries with special characters."""
        assert not is_url("What is Python?")
        assert not is_url("Rust vs Go")


class TestCheckLlmsTxt:
    """Test llms.txt checking."""

    @patch("subprocess.run")
    def test_llms_txt_found(self, mock_run):
        """Test when llms.txt is found."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout="# Documentation\n\nContent here",
            stderr=""
        )

        result = check_llms_txt("https://example.com")
        assert result is not None
        assert "Documentation" in result

    @patch("subprocess.run")
    def test_llms_txt_not_found(self, mock_run):
        """Test when llms.txt doesn't exist."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout="",
            stderr=""
        )

        result = check_llms_txt("https://example.com")
        assert result is None

    @patch("subprocess.run")
    def test_llms_txt_too_short(self, mock_run):
        """Test when llms.txt content is too short."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout="# Short",
            stderr=""
        )

        result = check_llms_txt("https://example.com")
        assert result is None


class TestFetchWithWebfetch:
    """Test webfetch integration."""

    @patch("subprocess.run")
    def test_webfetch_success(self, mock_run):
        """Test successful webfetch."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout="# Page Title\n\nSome content here",
            stderr=""
        )

        result = fetch_with_webfetch("https://example.com")
        assert result is not None
        assert result.source == "webfetch"
        assert "Page Title" in result.content

    @patch("subprocess.run")
    def test_webfetch_fails(self, mock_run):
        """Test webfetch failure."""
        mock_run.side_effect = Exception("Command failed")

        result = fetch_with_webfetch("https://example.com")
        assert result is None


class TestSearchWithWebsearch:
    """Test websearch integration."""

    @patch("subprocess.run")
    def test_websearch_success(self, mock_run):
        """Test successful websearch."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout="# Search Results\n\nResult 1: Some content",
            stderr=""
        )

        result = search_with_websearch("test query")
        assert result is not None
        assert result.source == "websearch"
        assert "Search Results" in result.content

    @patch("subprocess.run")
    def test_websearch_fails(self, mock_run):
        """Test websearch failure."""
        mock_run.side_effect = Exception("Command failed")

        result = search_with_websearch("test query")
        assert result is None


class TestResolveUrl:
    """Test URL resolution."""

    @patch("scripts.resolve.check_llms_txt")
    @patch("scripts.resolve.fetch_with_webfetch")
    @patch("scripts.resolve.search_with_websearch")
    def test_url_with_llms_txt(self, mock_search, mock_fetch, mock_llms):
        """Test URL with llms.txt."""
        mock_llms.return_value = "# llms.txt content"
        
        result = resolve_url("https://example.com")
        assert result["source"] == "llms.txt"
        assert "llms.txt content" in result["content"]
        mock_fetch.assert_not_called()

    @patch("scripts.resolve.check_llms_txt")
    @patch("scripts.resolve.fetch_with_webfetch")
    @patch("scripts.resolve.search_with_websearch")
    def test_url_fallback_to_webfetch(self, mock_search, mock_fetch, mock_llms):
        """Test URL fallback to webfetch."""
        mock_llms.return_value = None
        mock_fetch.return_value = ResolvedResult(source="webfetch", content="# Content")
        
        result = resolve_url("https://example.com")
        assert result["source"] == "webfetch"

    @patch("scripts.resolve.check_llms_txt")
    @patch("scripts.resolve.fetch_with_webfetch")
    @patch("scripts.resolve.search_with_websearch")
    def test_url_all_fail(self, mock_search, mock_fetch, mock_llms):
        """Test URL when all methods fail."""
        mock_llms.return_value = None
        mock_fetch.return_value = None
        mock_search.return_value = None
        
        result = resolve_url("https://example.com")
        assert result["source"] == "none"
        assert "error" in result


class TestResolveQuery:
    """Test query resolution."""

    @patch("scripts.resolve.search_with_websearch")
    def test_query_success(self, mock_search):
        """Test successful query resolution."""
        mock_search.return_value = ResolvedResult(
            source="websearch", content="# Results\nContent"
        )
        
        result = resolve_query("test query")
        assert result["source"] == "websearch"

    @patch("scripts.resolve.search_with_websearch")
    def test_query_fails(self, mock_search):
        """Test query when search fails."""
        mock_search.return_value = None
        
        result = resolve_query("test query")
        assert result["source"] == "none"
        assert "error" in result


class TestResolve:
    """Test main resolve function."""

    @patch("scripts.resolve.resolve_url")
    def test_resolve_url(self, mock_url):
        """Test resolve with URL input."""
        mock_url.return_value = {"source": "webfetch", "content": "test"}
        
        result = resolve("https://example.com")
        assert result["source"] == "webfetch"

    @patch("scripts.resolve.resolve_query")
    def test_resolve_query(self, mock_query):
        """Test resolve with query input."""
        mock_query.return_value = {"source": "websearch", "content": "test"}
        
        result = resolve("some search")
        assert result["source"] == "websearch"


class TestResolvedResult:
    """Test ResolvedResult dataclass."""

    def test_to_dict(self):
        """Test conversion to dictionary."""
        result = ResolvedResult(
            source="test",
            content="content",
            url="https://example.com",
            query=None,
            error=None,
            validated_links=["https://example.com/1"]
        )
        
        d = result.to_dict()
        assert d["source"] == "test"
        assert d["url"] == "https://example.com"
        assert d["validated_links"] == ["https://example.com/1"]


class TestConstants:
    """Test constants."""

    def test_max_chars(self):
        """Test MAX_CHARS default."""
        assert MAX_CHARS == 8000

    def test_min_chars(self):
        """Test MIN_CHARS default."""
        assert MIN_CHARS == 200


class TestEdgeCases:
    """Edge case tests."""

    def test_empty_string(self):
        """Test empty string handling."""
        assert not is_url("")
        assert not is_url("   ")

    def test_url_case(self):
        """Test URL scheme case handling."""
        assert is_url("HTTPS://EXAMPLE.COM")
        assert is_url("http://example.com")

    def test_url_with_port(self):
        """Test URLs with port numbers."""
        assert is_url("https://example.com:8080/path")
        assert is_url("http://localhost:3000/api")

    def test_url_with_query_params(self):
        """Test URLs with query parameters."""
        assert is_url("https://example.com/path?param=value")
        assert is_url("https://example.com/search?q=hello")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
