package io.provenance.example.util

import kotlin.system.exitProcess

/**
 * The amount of tries that indicates an infinite input loop.  Any value <= to this value will indicate infinite.
 */
const val TRIES_INFINITE = 0

/**
 * Additional parameters to be specified to the InputUtil's various input reading functions.
 *
 * @param default The default value to be used when the user declines input. Defaults to null, indicating user input is
 *                required.
 * @param validation A definition that will cause input to be rejected based on a set of rules.  Defaults to null, indicating
 *                   that validation does not need to be run and all input should be accepted if non-empty.
 * @param tries The amount of attempts to make to get valid user input.  A number equal to or less than zero will result
 *              in infinite attempts for user input.
 * @param exitAliases If the user enters any of these values (case insensitive), the program will immediately exit.
 */
data class InputParams<T>(
    val default: DefaultParam<T>? = null,
    val validation: InputValidation<T>? = null,
    val tries: Int = TRIES_INFINITE,
    val exitAliases: Collection<String>? = listOf("quit")
)

/**
 * The default value for an input function.  Defines the value itself, and the string to display for the value.
 *
 * @param value The value to use when the user declines to input a value and presses enter immediately.
 * @param description The description for the default value that is displayed to the user.  Use this field when running
 *                    .toString() on the value itself produces an unreadable or undesirable value.
 */
data class DefaultParam<T>(
    val value: T,
    val description: String = value.toString(),
)

/**
 * Defines if user input is valid based on a functional receiver for converted user input.
 *
 * @param validate The result of this contained function, when run against the derived value from user input, will be used to
 *                 determine if input is valid or not.  On false, input will rejected.  Defaults to true, indicating
 *                 that validation has passed.
 * @param validationRuleText A list of strings, defining each step taken during validation.  This is printed when the
 *                           user input is rejected for being invalid via the validator function.
 */
data class InputValidation<T>(
    val validate: (T) -> Boolean,
    val validationRuleText: List<String> = listOf("Input must be valid"),
)

/**
 * A utility to facilitate user input, with baked-in retries, default values, and the ability to exit the program with
 * a universal command.
 */
object InputUtil {

    /**
     * The core input function.  Displays a defined message to the user and attempts to process their commandline input
     * as the specified type.
     *
     * @param message The message to the user, explaining the type of input desired.
     * @param params Additional arguments to the function that modify its behavior.
     * @param converter A function that takes the user input and converts it to the type required.  If the resulting
     *                  value is null, the prompt will be retried.
     */
    fun <T> input(
        message: String,
        params: InputParams<T> = InputParams(),
        converter: (String) -> T?
    ): T {
        // The amount of attempts for user input so far
        var tryCounter = 0
        // If the amount of tries is <= 0, this indicates that the loop should be infinite until the user quits or enters
        // a valid value
        val infiniteRetries = params.tries <= TRIES_INFINITE
        while (infiniteRetries || tryCounter < params.tries) {
            try {
                // Prints output based on the input params and the current retry.  Also displays the default value, if
                // provided in the params.
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
                // If valid user input was accepted, attempt to parse it through the converter
                val result = input?.let(converter)
                    // Otherwise, use the default
                    ?: params.default?.value
                    // Bad case: blow up and try again, letting the user
                    ?: throw IllegalStateException("Could not parse input from entered value")
                // If validation is non-null and the validate function fails to produce a true, reject user input with
                // an exception and print rule text
                if (params.validation?.validate?.invoke(result) == false) {
                    throw IllegalStateException("Input failed validation: ${params.validation.validationRuleText}")
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

    /**
     * A helper function to fetch a string from the input function without being forced to specify a converter.
     */
    fun inputString(message: String, params: InputParams<String> = InputParams()): String = input(
        message = message,
        params = params,
        // Take user input as-is
        converter = { it }
    )

    /**
     * A helper function to fetch an enum from user input.  Requires that the user enters one of the specified enum types,
     * dynamically derived from the class itself.
     */
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

