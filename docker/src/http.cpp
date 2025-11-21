#include "http.h"

Curl::Curl()
    : m_handle(nullptr)
{
    CURLcode result = curl_global_init(CURL_GLOBAL_NOTHING);
    if (result != CURLE_OK) {
        throw new CurlError("curl_global_init failed", result);
    }

    CURL* handle = curl_easy_init();
    if (handle == nullptr) {
        throw new CurlErrorBase("curl_easy_init failed");
    }
}

std::string Curl::get(const std::string& url, const HeaderMap& headers) {
    // CURLcode curl_easy_setopt(CURL *handle, CURLOPT_HTTPGET, long useget);

    CURLcode result = curl_easy_setopt(m_handle, CURLOPT_URL, url.c_str());
    if (result != CURLE_OK) {
        throw new CurlError("curl_easy_setopt (CURLOPT_URL) failed", result);
    }

    std::string response;
    response.reserve(1024 * 1024);
    result = curl_easy_setopt(m_handle, CURLOPT_WRITEFUNCTION, [&response] (
        char* ptr,
        size_t _size,
        size_t nmemb,
        void* _userdata
    ) -> size_t {
        // Documentation states this function may be called with 0 bytes if the transferred file is empty.
        if (nmemb != 0) {
            response.append(ptr, nmemb);
        }

        return nmemb;
    });
    
    // TODO: Pick up here.
    // CURLcode curl_easy_setopt(CURL *handle, CURLOPT_HTTPHEADER,
                          // struct curl_slist *headers);
    CurlStringList headers;
    for (const auto& [key, value] : headers) {
        std::ostringstream header;
        header << key << ": " << value;
        headers.append(header);
    }
    result = curl_easy_setopt(m_hanlde, CURLOPT_HTTPHEADER, headers.native());
    if (result != CURLE_OK) {
        throw new CurlError("curl_easy_setopt (CURLOPT_HTTPHEADER) failed");
    }

    // success = curl_easy_perform(handle);
}

std::ostream& operator>>(std::ostream& os, const CurlErrorBase& error) {
    os << "CURL: " << error.message();
}

std::ostream& operator>>(std::ostream& os, const CurlError& error) {
    os << "CURL:" << error.message() << ", error code = " << error.code();
}

void CurlStringList::append(const std::string& value) {
    curl_slist* new_list = curl_slist_append(m_slist, value.c_str());
    if (new_list == nullptr) {
        throw new CurErrorBase("curl_slist_append failed");
    }
    m_slist = new_list;
}