package nl.openweb.commandhandler

import nl.openweb.data.AccountCreationConfirmed
import nl.openweb.data.ConfirmAccountCreation
import org.apache.kafka.streams.kstream.KStream
import org.springframework.cloud.stream.annotation.Input
import org.springframework.cloud.stream.annotation.Output

interface AccountCreation {
    @Input(CAC)
    fun input(): KStream<String, ConfirmAccountCreation>

    @Output(ACCO)
    fun fail(): KStream<String, AccountCreationConfirmed>

    @Output(ACFA)
    fun succeed(): KStream<String, AccountCreationConfirmed>

    companion object {
        const val CAC = "cac"
        const val ACCO = "acco"
        const val ACFA = "acfa"
    }
}