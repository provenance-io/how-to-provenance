package io.provenance.example.util

const val TRIES_INFINITE = 0

data class InputParams(
    val rejectEmptyInput: Boolean = true,
    val tries: Int = TRIES_INFINITE,
)

inline fun <reified T> captureInput(
    message: String,
    params: InputParams = InputParams(),
    converter: (String) -> T?
): T {
    var tryCounter = 0
    val infiniteRetries = params.tries <= TRIES_INFINITE
    while (infiniteRetries || tryCounter < params.tries) {
        try {
            return print("${if (tryCounter > 0) "[RETRY $tryCounter]" else ""}$message ")
                .run { readLine() }
                .takeIf { !params.rejectEmptyInput || !it.isNullOrBlank() }
                .elvis { throw IllegalStateException("Expected input to be non-null") }
                .let(converter)
                .elvis { throw IllegalStateException("Expected type [${T::class.qualifiedName}] could not be parsed from input") }
        } catch (e: Exception) {
            tryCounter++
            if (!infiniteRetries && tryCounter == params.tries) {
                throw e
            } else {
                println(e.message)
                continue
            }
        }
    }
    throw IllegalStateException("Attempt count [$params.tries] violated")
}

fun captureString(message: String, params: InputParams = InputParams()): String = captureInput(message) { it }

inline fun <reified T: Enum<T>> captureEnumOrNull(
    messagePrefix: String? = null,
    params: InputParams = InputParams()
): T = enumValues<T>().let { values ->
    captureInput(
        message = "${messagePrefix?.let { "$it " } ?: "" }Please enter one of ${enumValues<T>().map { it.name }}:",
        converter = { input -> values.firstOrNull { it.name.lowercase() == input.lowercase() } },
    )
}


