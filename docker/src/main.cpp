#include <iostream>

#include "http.h"

constexpr char* IMAGE_NAME = "nginx";

int main(int argc, char** argv) {
    Curl curl;
    std::string token = curl.get("https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/nginx:pull");

    return 0;
}