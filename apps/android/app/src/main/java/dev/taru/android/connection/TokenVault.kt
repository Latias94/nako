package dev.taru.android.connection

interface TokenVault {
    fun saveToken(reference: String, token: String)
    fun readToken(reference: String): String?
    fun deleteToken(reference: String)
}

class InMemoryTokenVault : TokenVault {
    private val tokens = linkedMapOf<String, String>()

    override fun saveToken(reference: String, token: String) {
        tokens[reference] = token
    }

    override fun readToken(reference: String): String? = tokens[reference]

    override fun deleteToken(reference: String) {
        tokens.remove(reference)
    }
}
