#pragma once

#include <optional>

#include <curl/curl.h>

#include "pch.h"

using HeaderMap = std::unordered_map<std::string, std::string>;

class CurlInit {
public:
    CurlInit();
    ~CurlInit() { curl_global_cleanup(); }

    CurlInit(const Curl&) = delete;
    CurlInit& operator=(const Curl&) = delete;
    Curl(Curl&&) = delete;
    Curl& operator=(Curl&&) = delete;
};

class Curl {
public:
    CurlHandle(CURL* handle);
    ~Curl() { release(); }

    Curl(const Curl&) = delete;
    Curl& operator=(const Curl&) = delete;

    Curl(Curl&& other) {
        m_handle = other.m_handle;
        other.m_handle = nullptr;
    }
    Curl& operator=(Curl&& other) {
        if (&other != this) {
            release();
            m_handle = other.m_handle;
            other.m_handle = nullptr;
        }
        return *this;
    }

    std::string get(const std::string& url, const HeaderMap& headers);

private:
    void release() { curl_global_cleanup(); }

    static CurlInit m_init;
    static std::once_flag m_once_init;
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
    ~CurlStringList() { free_list(); }

    CurlStringList(const CurlStringList&) = delete;
    CurlStringList& operator=(const CurlStringList&) = delete;

    CurlStringList(CurlStringList&& other) {
        m_slist = other.m_slist;
        other.m_slist = nullptr;
    }
    CurlStringList& operator=(CurlStringList&& other) {
        if (&other != this) {
            free_list();
            m_slist = other.m_slist;
            other.m_slist = nullptr;
        }
        return *this;
    }

    void append(const std::string& value);
    curl_slist* native() const { return m_slist; }

private:
    void free_list() {
        curl_slist_free_all(m_slist);
        m_slist = nullptr;
    }

    curl_slist* m_slist;
};