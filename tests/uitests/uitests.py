#!/usr/bin/env python3

import os

import unittest
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.wait import WebDriverWait
from selenium.webdriver.firefox.options import Options

firefox_binary = os.environ.get("FIREFOX_BINARY")

class SearchTest(unittest.TestCase):
    def setUp(self):
        options = Options()
        options.add_argument("--headless")
        options.binary_location = firefox_binary
        self.browser = webdriver.Firefox(options=options)
        self.addCleanup(self.browser.quit)

        self.base_url = "http://localhost/"

    def test_simple_search(self):
        self.browser.get(self.base_url)

        query = self.browser.find_element(By.ID, "query")
        query.send_keys("SimpleSearch")

        content = self.browser.find_element(By.ID, "content")

        wait = WebDriverWait(self.browser, timeout=5)
        wait.until(lambda _: "Core code (1 lines" in content.text)
        wait.until(lambda _: "class SimpleSearch" in content.text)

        wait.until(lambda _: "SimpleSearch" in self.browser.current_url)

    def test_case_sensitiveness(self):
        self.browser.get(self.base_url)

        query = self.browser.find_element(By.ID, "query")
        query.send_keys("CaseSensitiveness")

        content = self.browser.find_element(By.ID, "content")

        wait = WebDriverWait(self.browser, timeout=5)
        wait.until(lambda _: "Core code (2 lines" in content.text)
        wait.until(lambda _: "class CaseSensitiveness1" in content.text)
        wait.until(lambda _: "class casesensitiveness2" in content.text)

        wait.until(lambda _: "CaseSensitiveness" in self.browser.current_url)
        wait.until(lambda _: "case=false" in self.browser.current_url)

        case_checkbox = self.browser.find_element(By.ID, "case")
        case_checkbox.click()

        wait.until(lambda _: "Core code (1 lines" in content.text)
        wait.until(lambda _: "class CaseSensitiveness1" in content.text)

        wait.until(lambda _: "CaseSensitiveness" in self.browser.current_url)
        wait.until(lambda _: "case=true" in self.browser.current_url)

    def test_regexp(self):
        self.browser.get(self.base_url)

        query = self.browser.find_element(By.ID, "query")
        query.send_keys("Simpl.Search")

        content = self.browser.find_element(By.ID, "content")

        wait = WebDriverWait(self.browser, timeout=5)
        wait.until(lambda _: "No results for current query" in content.text)

        wait.until(lambda _: "Simpl.Search" in self.browser.current_url)
        wait.until(lambda _: "regexp=false" in self.browser.current_url)

        regexp_checkbox = self.browser.find_element(By.ID, "regexp")
        regexp_checkbox.click()

        wait.until(lambda _: "Core code (1 lines" in content.text)
        wait.until(lambda _: "class SimpleSearch" in content.text)

        wait.until(lambda _: "Simpl.Search" in self.browser.current_url)
        wait.until(lambda _: "regexp=true" in self.browser.current_url)

    def test_path_filter(self):
        self.browser.get(self.base_url)

        query = self.browser.find_element(By.ID, "query")
        query.send_keys("PathFilter")

        content = self.browser.find_element(By.ID, "content")

        wait = WebDriverWait(self.browser, timeout=5)
        wait.until(lambda _: "Core code (2 lines" in content.text)
        wait.until(lambda _: "class PathFilter" in content.text)
        wait.until(lambda _: "UITests.cpp" in content.text)
        wait.until(lambda _: "UITestsPathFilter.cpp" in content.text)

        wait.until(lambda _: "PathFilter" in self.browser.current_url)
        wait.until(lambda _: "path=&" in self.browser.current_url)

        query = self.browser.find_element(By.ID, "path")
        query.send_keys("Filter.cpp")

        wait.until(lambda _: "Core code (1 lines" in content.text)
        wait.until(lambda _: "class PathFilter" in content.text)
        wait.until(lambda _: "UITests.cpp" not in content.text)
        wait.until(lambda _: "UITestsPathFilter.cpp" in content.text)

        wait.until(lambda _: "PathFilter" in self.browser.current_url)
        wait.until(lambda _: "path=Filter.cpp&" in self.browser.current_url)

unittest.main(verbosity=2)
