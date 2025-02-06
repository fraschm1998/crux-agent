package me.fraschetti.agent.utils

import com.novi.serde.Bytes
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.headers
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.http.HttpMethod
import io.ktor.util.flattenEntries
import me.fraschetti.agent.shared_types.HttpHeader
import me.fraschetti.agent.shared_types.HttpRequest
import me.fraschetti.agent.shared_types.HttpResponse

suspend fun requestHttp(
    client: HttpClient,
    request: HttpRequest,
): HttpResponse {
    val response =
        client.request(request.url) {
            timeout {
                requestTimeoutMillis = 60000 // 60 seconds
                connectTimeoutMillis = 60000 // 60 seconds
            }
            this.method = HttpMethod(request.method)
            this.headers {
                for (header in request.headers) {
                    append(header.name, header.value)
                }
            }
            // Add the request body if it exists
            request.body?.let { requestBody ->
                this.setBody(requestBody.content())
            }
        }
    val bytes = Bytes.valueOf(response.body())
    val headers = response.headers.flattenEntries().map { HttpHeader(it.first, it.second) }
    return HttpResponse(response.status.value.toShort(), headers, bytes)
}
