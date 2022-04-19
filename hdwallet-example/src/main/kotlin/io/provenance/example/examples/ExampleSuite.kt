package io.provenance.example.examples

/**
 * A simple interface that ensures each implementation has an entrypoint.  For chaining input in the main() function
 * of the application.
 */
interface ExampleSuite {
    fun start()
}
