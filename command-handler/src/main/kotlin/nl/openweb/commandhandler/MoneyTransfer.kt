package nl.openweb.commandhandler

import nl.openweb.data.BalanceChanged
import nl.openweb.data.ConfirmMoneyTransfer
import nl.openweb.data.MoneyTransferConfirmed
import nl.openweb.data.MoneyTransferFailed
import org.apache.kafka.streams.kstream.KStream
import org.springframework.cloud.stream.annotation.Input
import org.springframework.cloud.stream.annotation.Output

interface MoneyTransfer {
    @Input(CMT)
    fun input(): KStream<String, ConfirmMoneyTransfer>

    @Output(MTCO)
    fun succeed(): KStream<String, MoneyTransferConfirmed>

    @Output(MTFA)
    fun failed(): KStream<String, MoneyTransferFailed>

    @Output(BACH)
    fun balance(): KStream<String, BalanceChanged>

    companion object {
        const val CMT = "cmt"
        const val MTCO = "mtco"
        const val MTFA = "mtfa"
        const val BACH = "bach"
    }
}