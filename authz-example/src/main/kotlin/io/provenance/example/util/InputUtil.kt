package io.provenance.example.util

import kotlin.system.exitProcess

const val TRIES_INFINITE = 0

data class InputParams<T>(
    val default: DefaultParam<T>? = null,
    val validation: (T) -> Boolean = { true },
    val tries: Int = TRIES_INFINITE,
    val exitAliases: Collection<String>? = listOf("quit")
)

data class DefaultParam<T>(
    val value: T,
    val description: String = value.toString(),
)

object InputUtil {
    fun <T> input(
        message: String,
        params: InputParams<T> = InputParams(),
        converter: (String) -> T?
    ): T {
        var tryCounter = 0
        val infiniteRetries = params.tries <= TRIES_INFINITE
        while (infiniteRetries || tryCounter < params.tries) {
            try {
                print(
                    (if (tryCounter > 0) "[RETRY $tryCounter] " else "") +
                            message +
                            (if (params.default != null) " (DEFAULT: ${params.default.description})" else "") +
                            ": "
                )
                val input = readLine().takeIf { !it.isNullOrBlank() }
                // If the user wants to quit the program, any of the exit aliases will suffice
                if (params.exitAliases != null && input?.lowercase() in params.exitAliases) {
                    exitProcess(0)
                }
                val result = input?.let(converter)
                    ?: params.default?.value
                    ?: throw IllegalStateException("Invalid input specified")
                if (!params.validation.invoke(result)) {
                    throw IllegalStateException("Invalid input!")
                }
                return result
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
        throw IllegalStateException("Attempt count [${params.tries}] violated")
    }

    fun inputString(message: String, params: InputParams<String> = InputParams()): String = input(message, params) { it }

    inline fun <reified T: Enum<T>> inputEnum(
        messagePrefix: String? = null,
        params: InputParams<T> = InputParams()
    ): T = enumValues<T>().let { values ->
        input(
            message = "${messagePrefix?.let { "$it " } ?: "" }Please enter one of ${enumValues<T>().map { it.name }}",
            params = params,
            converter = { input -> values.firstOrNull { it.name.lowercase() == input.lowercase() } },
        )
    }
}

