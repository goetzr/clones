#pragma once

#include <optional>

#include <curl/curl.h>

#include "pch.h"

class Curl {
public:
    Curl();
    ~Curl() { curl_global_cleanup(); }
    std::string get(const std::string& url);

private:
    CURL* m_handle;
};

class CurlError {
public:
    CurlError(const std::string& message)
        : message{message},
          code{std::nullopt}
    {}

    CurlError(const std::string& message, CURLcode code)
        : message{message},
          code{code}
    {}

private:
    std::string message;
    std::optional<CURLcode> code;

    friend std::ostream& operator>>(std::ostream& os, const CurlError& error);
};

