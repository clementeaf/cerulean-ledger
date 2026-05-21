Lo que cerulean-ledger necesita implementar:                                                                                                                                                            
                                                                                                                                                                                                          
  1. Alias Registry (storage + endpoints)     
                                                                                                                                                                                                          
  Storage: nuevo módulo alias_registry con estructura:                                                                                                                                                    
  commitment (SHA-256, 64 hex chars) → {                                                                                                                                                                  
    did: String,                                                                                                                                                                                          
    salt: String (32 hex chars),                                                                                                                                                                          
    encrypted_alias: String,                                                                                                                                                                              
    registered_at: u64,                                                                                                                                                                                   
    status: "active" | "revoked",                                                                                                                                                                         
    revoked_at: Option<u64>                                                                                                                                                                               
  }                                                                                                                                                                                                       
                                                                                                                                                                                                          
  Endpoints:                                                                                                                                                                                              
  - POST /api/v1/alias/register — body: { did, commitment, salt, encrypted_alias, signature }. Rechazar 409 si commitment ya existe.
  - POST /api/v1/alias/resolve — body: { commitment }. Devolver { did, address } o 404. Rate limit: 10/min por IP.                                                                                        
  - POST /api/v1/alias/revoke — body: { did, commitment, signature }. Marcar status: "revoked", revoked_at: now. Tras 15 días, commitment se libera para re-registro.
                                                                                                                                                                     
  Reglas:                                                                                                                                                                                                 
  - Un DID = un alias activo máximo           
  - Commitment duplicado = 409                                                                                                                                                                            
  - Alias revocado: 15 días bloqueado, después se puede re-registrar por cualquiera                                                                                                                       
  - Verificar signature Ed25519 del DID en register y revoke                                                                                                                                              
                                                                                                                                                                                                          
  2. Invitation System (endpoints)                                                                                                                                                                        
                                                                                                                                                                                                          
  Endpoints:                                                                                                                                                                                              
  - POST /api/v1/governance/invitations — body: { from_did, to_commitment, proposal_ids, signature }. Crear invitación vinculada al commitment destino.                                                   
  - GET /api/v1/governance/invitations?voter={address} — ya existe stub, devolver invitaciones pendientes.                                                                                                
  - POST /api/v1/governance/invitations/respond — body: { invitation_id, accepted, signature }.                                                                                                           
                                                                  
  Reglas:                                                                                                                                                                                                 
  - Max 20 proposal_ids por invitación (anti-spam)
  - Max 5 invitaciones por hora por from_did (rate limit)                                                                                                                                                 
  - Verificar firma Ed25519 en todas las operaciones                                                                                                                                                      
                                                                                                                                                                                                          
  3. Futuro: Event Bus                                                                                                                                                                                    
                                                                  
  Reemplazar REST por NATS para alias ops. Pero eso es fase 4 — los REST endpoints son suficientes para arrancar. 