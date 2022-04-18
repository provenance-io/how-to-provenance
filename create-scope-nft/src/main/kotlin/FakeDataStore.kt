import io.provenance.scope.util.sha256String

class FakeDataStore {
    private val hashMap = hashMapOf<String, ByteArray>()

    /**
     * Add an item to storage, keyed by base-encoded sha256
     * @param data: the data to store
     * @returns the base64-encoded sha256 of the data (key of the data in storage)
     */
    fun put(data: ByteArray): String = data.sha256String().also { hashMap.put(it, data) }

    /**
     * Retireve an item from storage by base64-encoded sha256 hash
     * @param hash: the base64-encoded sha256 of the data
     * @returns the data stored at the hash
     */
    fun get(hash: String) = hashMap.get(hash) ?: throw IllegalStateException("item at hash $hash not found in storage")
}
