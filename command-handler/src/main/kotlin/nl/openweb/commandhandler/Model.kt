package nl.openweb.commandhandler

import java.time.LocalDateTime
import java.util.*
import javax.persistence.Entity
import javax.persistence.GeneratedValue
import javax.persistence.Id
import javax.persistence.Table

@Entity
@Table(name = "balance")
data class Balance(@Id @GeneratedValue
                   val balanceId: Int,
                   val iban: String,
                   val token: String,
                   val amount: Long,
                   val type: String,
                   val lmt: Long,
                   val createdAt: LocalDateTime,
                   val updatedAt: LocalDateTime)

@Entity
@Table(name = "cac")
data class Cac(@Id
               val uuid: UUID,
               val iban: String,
               val token: String,
               val type: String,
               val reason: String,
               val createdAt: LocalDateTime)

@Entity
@Table(name = "cmt")
data class Cmt(@Id
               val uuid: UUID,
               val reason: String,
               val createdAt: LocalDateTime)