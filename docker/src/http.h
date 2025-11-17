#pragma once

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

enum class CurlErrorKind {
    Code,
    Message
};

class CurlError {
public:
    CurlError(CURLcode code)
        : kind(CurlErrorKind::Code),
          code{code}
    {}

    CurlError(const std::string& message)
        : kind(CurlErrorKind::Message),
          message(message)
    {}

private:
    CurlErrorKind kind;
    union {
        CURLcode code;
        std::string message;
    };

    friend std::ostream& operator>>(std::ostream& os, const CurlError& error);
};

