package nl.openweb.commandhandler

import nl.openweb.commandhandler.Utils.toUuid
import nl.openweb.data.AccountCreationConfirmed
import nl.openweb.data.AccountCreationFailed
import nl.openweb.data.ConfirmAccountCreation
import org.apache.avro.specific.SpecificRecord
import org.apache.kafka.streams.kstream.KStream
import org.apache.kafka.streams.kstream.Predicate
import org.springframework.boot.autoconfigure.EnableAutoConfiguration
import org.springframework.cloud.stream.annotation.EnableBinding
import org.springframework.cloud.stream.annotation.StreamListener
import org.springframework.data.repository.findByIdOrNull
import org.springframework.messaging.handler.annotation.SendTo
import java.time.LocalDateTime
import java.util.*

@EnableBinding(AccountCreation::class)
@EnableAutoConfiguration
class AccountCreationProcessor(private val balanceRepository: BalanceRepository,
                               private val cacRepository: CacRepository) {

    @StreamListener(AccountCreation.CAC)
    @SendTo(AccountCreation.ACCO, AccountCreation.ACFA)
    fun process(input: KStream<String, ConfirmAccountCreation>): Array<KStream<String, SpecificRecord>>{
        val isSucces = Predicate{_: String, v: SpecificRecord -> v is AccountCreationConfirmed}
        val isFail = Predicate{_: String, v: SpecificRecord -> v is AccountCreationFailed}
        return input
                .mapValues(this::getResponse)
                .branch(isSucces, isFail)
    }

    private fun getResponse(cac: ConfirmAccountCreation): SpecificRecord {
        val cacId = cac.id.toUuid()
        return cacRepository.findByIdOrNull(cacId)?.let{
            if (it.iban.isBlank()) {
                AccountCreationFailed(cac.id, it.reason)
            } else {
                AccountCreationConfirmed(cac.id, it.iban, it.token, cac.aType)
            }
        } ?: tryToGenerateAccount(cac, cacId)
    }

    private fun tryToGenerateAccount(cac: ConfirmAccountCreation, cacId: UUID) : SpecificRecord{
        val iban = Utils.getIban()
        val balance = balanceRepository.findByIban(iban)
        val now = LocalDateTime.now()
        return if (balance.isEmpty()) {
            val token = Utils.getToken()
            cacRepository.save(Cac(cacId, iban, token, cac.aType.toString(), "", now))
            balanceRepository.save(Balance(0, iban, token, 0L, cac.aType.toString(), -50000L, now, now))
            AccountCreationConfirmed(cac.id, iban, token, cac.aType)
        } else {
            val reason = "generated iban already exists, try again"
            cacRepository.save(Cac(cacId, "", "", cac.aType.toString(), reason, now))
            AccountCreationFailed(cac.id, reason)
        }
    }
}
