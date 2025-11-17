#include "http.h"

Curl::Curl()
    : m_handle(nullptr)
{
    CURLcode result = curl_global_init(CURL_GLOBAL_NOTHING);
    if (result != CURLE_OK) {
        throw new CurlError(result);
    }

    CURL* handle = curl_easy_init();
    if (handle == nullptr) {
        throw new CurlError("curl_easy_init failed");
    }
}

std::string Curl::get(const std::string& url) {
    // TODO: Code error should include string description.
    CURLcode result = curl_easy_setopt(m_handle, CURLOPT_URL, url.c_str());
    if (result != CURLE_OK) {
        throw new CurlError(result);
    }
}

std::ostream& operator>>(std::ostream& os, const CurlError& error) {
    switch(error.kind) {
        case CurlErrorKind::Code:
            os << "error code: " << error.code;
            break;
        case CurlErrorKind::Message:
            os << error.message;
            break;
    };
}