#pragma once

#include <optional>

#include <curl/curl.h>

#include "pch.h"

using HeaderMap = std::unordered_map<std::string, std::string>;

class Curl {
public:
    Curl();
    ~Curl() { curl_global_cleanup(); }

    std::string get(const std::string& url, const HeaderMap& headers);

private:
    CURL* m_handle;
};

class CurlErrorBase {
public:
    explicit CurlErrorBase(const std::string& message)
        : m_message{message}
    {}

    const std::string& message() const { return m_message; }

protected:
    std::string m_message;
};

class CurlError : public CurlErrorBase {
public:
    CurlError(const std::string& message, CURLcode code)
        : CurlErrorBase{message},
          m_code{code}
    {}

    CURLcode code() const { return m_code; }

private:
    CURLcode m_code;
};

class CurlStringList {
public:
    CurlStringList() : m_slist{nullptr} {};
    ~CurlStringList() { curl_slist_free_all(m_slist); m_slist = nullptr; }

    void append(const std::string& value);
    curl_slist* native() const { return m_slist; }
private:
    curl_slist* m_slist;
};