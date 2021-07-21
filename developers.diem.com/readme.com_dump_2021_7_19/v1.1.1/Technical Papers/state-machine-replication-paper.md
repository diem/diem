---
title: "State Machine Replication"
slug: "state-machine-replication-paper"
hidden: false
metadata: 
  title: "State Machine Replication"
  description: "This technical paper describes the Diem Byzantine Fault Tolerance (DiemBFT) algorithmic core and discusses next steps in its production."
createdAt: "2021-02-04T01:06:43.783Z"
updatedAt: "2021-03-30T03:45:58.922Z"
---
***Note to readers: On December 1, 2020, the Libra Association was renamed to Diem Association. This report was modified in April 2020 to incorporate updates to the Libra payment system as found in the White Paper v2.0. Features of the project as implemented may differ based on regulatory approvals or other considerations, and may evolve over time.***

## Abstract

This report describes the Diem Byzantine Fault Tolerance (DiemBFT) algorithmic core and discusses next steps in its production. The consensus protocol is responsible for forming agreement on ordering and finalizing transactions among a configurable set of validators. DiemBFT maintains safety against network asynchrony and even if at any particular configuration epoch, a threshold of the participants are Byzantine.

DiemBFT is based on HotStuff, a recent protocol that leverages several decades of scientific advances in Byzantine Fault Tolerance (BFT) and achieves the strong scalability and security properties required by internet settings. Several novel features distinguish DiemBFT from HotStuff. DiemBFT incorporates a novel round synchronization mechanism that provides bounded commit latency under synchrony. It introduces a nil-block vote that allows proposals to commit despite having faulty leaders. It encapsulates the correct behavior by participants in a “tcb”-able module, allowing it to run within a secure hardware enclave that reduces the attack surface on participants.

DiemBFT can reconfigure itself, by embedding configuration-change commands in the sequence. A new configuration epoch may change everything from the validator set to the protocol itself.

### Downloads
[block:html]
{
  "html": "<d-publication-link \n  image=\"https://diem-developers-components.netlify.app/images/state-machine-pdf.png\"\n  doc-link=\"https://diem-developers-components.netlify.app/papers/diem-consensus-state-machine-replication-in-the-diem-blockchain/2020-05-26.pdf\"\n  title=\"State Machine Replication in the Diem Blockchain\"\n></d-publication-link>"
}
[/block]