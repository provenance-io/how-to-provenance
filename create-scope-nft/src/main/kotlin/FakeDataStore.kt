import io.provenance.scope.util.sha256String

class FakeDataStore {
    private val hashMap = hashMapOf<String, ByteArray>()

    fun put(data: ByteArray) = hashMap.put(data.sha256String(), data)

    fun get(hash: String) = hashMap.get(hash)
}
