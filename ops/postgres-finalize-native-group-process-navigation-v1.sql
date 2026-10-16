-- Generated isolated-fixture-only atomic Group navigation correction.
-- Caller freezes/verifies actual database name/OID/system_identifier, owns the
-- cluster schema/role maintenance lease, BEGINs READ COMMITTED, sets LOCAL
-- search_path=pg_catalog,pg_temp, jit=off and positive bounded timeouts in a
-- separate statement, locks the ledger SHARE and actual console_account_owner
-- pg_authid row FOR UPDATE. Retain all locks through COMMIT/ROLLBACK. Role-row
-- custody is a trusted caller obligation; no fabricated pg_locks tuple proof.
-- This fixture protocol does not authorize production DDL or client exposure.
DO $native_group_process_navigation_custody$
DECLARE observed_phase text; variant_name text; expected_variant text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user IS DISTINCT FROM current_user OR current_user<>'console_buck_admin'
  OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) IS NOT TRUE
  OR starts_with(current_database(),'_sqlx_test_') IS NOT TRUE
  OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)
      IS DISTINCT FROM 'buck-sqlx-superuser-v1' THEN
  RAISE EXCEPTION 'native_group_process_navigation.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR pg_catalog.current_setting('search_path') IS DISTINCT FROM 'pg_catalog, pg_temp'
  OR pg_catalog.current_setting('jit') IS DISTINCT FROM 'off' THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_settings_mismatch'; END IF;
 IF (pg_catalog.current_setting('lock_timeout') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='lock_timeout')
      BETWEEN 1 AND 1000) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;
 IF (pg_catalog.current_setting('statement_timeout') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='statement_timeout')
      BETWEEN 1 AND 60000) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;
 IF (pg_catalog.current_setting('idle_in_transaction_session_timeout') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='idle_in_transaction_session_timeout')
      BETWEEN 1 AND 30000) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;
 IF (pg_catalog.current_setting('transaction_timeout') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='transaction_timeout')
      BETWEEN 1 AND 120000) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;
 IF (SELECT count(*)=1 AND bool_and(c.oid IS NOT NULL AND c.oid>0
      AND c.relkind='r' AND NOT c.relispartition AND r.rolname='console_app') IS TRUE
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
     JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
     WHERE n.nspname='public' AND c.relname='_sqlx_migrations') IS NOT TRUE
  OR (SELECT count(*)=1 FROM pg_catalog.pg_locks
      WHERE pid=pg_backend_pid() AND locktype='relation'
       AND relation=pg_catalog.to_regclass('public._sqlx_migrations')
       AND mode='ShareLock' AND granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_lock_missing'; END IF;
 IF (WITH expected_migrations(version,checksum) AS (
VALUES
 (1,'3a0113a4ccfa33c60918f873847653d4a23ca6a8e526e2b7389e7138f7041823901b9fbefaffaae3a056da8878791ed8'),
 (2,'4e7a83b1e764bb4ed9a1a18e8cc758f8b3343eacc12db6430a1aca17828ad3ed940f0d7d8b983a01eb9843be363ec32d'),
 (3,'43d4988151f9bd76bf9cd5f4306d25e41450efab4d8c73738839fd3c530dd91f547608b3327fd962640460595523e0cb'),
 (4,'b0677f4d9f7c52dd41b26b87e0ea8cdaa37ea128209e37649f15607034c5ddfe4c7fa4b60a52f5aedf7a908c56de3854'),
 (5,'4992bd5ed0da6dad0078d5b828027ae7a793c7b46add0a339ee5032a50ee1c618963348380085c20faaad592cc275bf0'),
 (6,'1d7327a051b404ae0578a79563a5d6a10590031e1d41d228cf7d64074192a40a6f0dbfd20b300e16b9cd1f0f87944115'),
 (7,'a1da5931f3b981ffe938dd12ea15f1c2a5a00a1122d3e767619752faea2d5e5e4befab42ce7c5ede4dc783754370d10a'),
 (8,'70e807747577343acd0ef1cd4efa36f279103442ad3a0a102eeed007da8d0c78fa414cc68623eda8a50e58806820737a'),
 (9,'289abe2c3fec02a0173020595f01301629c988272599e40ec0edaf4fc974189cdb267721fa01e27cbeeda38a4133328b'),
 (10,'145a8b5ee1ddda01ffbbdf7feb43a5b9b9a4a2149439384c0a6e049783fec6d9b25df84d517a2b0b21749ba29c42e077'),
 (11,'5e06932f48473ca6a6d9c02eead6ffcb78225678339edfa3c33ca3dc0d3a054d681dd92189e249389c8fb8c60b96d905'),
 (12,'2e7c57244bf658c89fc0b5be1e3ef423ece04afa7f4525040680e1ab0adfd4fb83c546031919feebf4f862df979ab505'),
 (13,'60d70cfa40d065e2369883ae2e19af771e69b491ad4f60432b1ed24f295bd6edda1323b333367bb0575b0fd3aa215b6d'),
 (14,'86ca8bc4ac10f840f1ebf27ff6f6e3a46682907f109e08a328fda2b84358bf8d9e6c307af83252aa4acf9353f3ea7852'),
 (15,'84523959c833801d4b094291b027c3f00d34f275a70bdb78467746d58f9515b84b3549e8df6bbe8f9f51a60216895b2b'),
 (16,'6fe1588630f882a6a9b0d834e9c1d7c781017b0970d78c0f00f239692cec4c20d1b5498f6d8ad0ca975f9435c828b11c'),
 (17,'efb61edac50e7ecb09814387cb141f0a4216e0bc606a17ee72ec8040990db06eef81100e1aaa87d4a999bbd8bd17f68b'),
 (18,'397304673b58370838ab79d3b6e9ab34e5caca708b6eff4c74defbbb3d9745fc61cfd5737d3f8d2345ba3fb08663f214'),
 (19,'6ba9d0fbd0f6fdbe57b956bf2536082e6f29cd9a7fc0c2d7e2988c9b0ad2226be4203fb944723ae48b077b4843b840e1'),
 (20,'b972a6fea540884bf715346245c7176af71ab3e7937c110a41d41c0eef7e83893b89fcc6e3311dd93271cd9c7d9f66b6'),
 (21,'de6c041eeb6885ebd5197c8bcda42746741b48dd2e288efd77086c516d7689808cd5ccecfa9d06959dd47721bbca7be0'),
 (22,'4d6d9a4fdaf749c6c963189184ac6898e21aded39fc8a1110fcf8a444bba062344d67210fe4285e4bf833541659234c2'),
 (23,'088767a033071d4beddf08ec887d08b99aee8f89aeae7719873af22c1d5238d42efd8737a60ceeb420962545583e78af'),
 (24,'b0dff6fca947bef8c1522f3e2eaf0a2d08bf9615639f5d20ea66ce1fd4feedb14cc56ac2c40a6a3dd1350b287b3ee241'),
 (25,'f5e6d37e2f1fd3e860a49efc1781712aa9a8033ce1f0373aa5825d77d67739eece75fef9b15e2ee0c302c7f247c74a66'),
 (26,'8fe15c2aad9d9758f63d71bfe89d292e1d37992e7a11e4f2203a8acb79004bb45a25e8d425c542e2df17433b12c227b4'),
 (27,'cb7514a726fac887ff29aaebfb109d6936f96b0f36afe64d4f16fb37881c307265434ff7bd04ca6c303e695738974b1f'),
 (28,'53ff45d0502e254e32bbe254390746cdb42a0cfa412a6769db58f20c2de6075bf08a328c10051f1b62fb92618e097c15'),
 (29,'f286d89243c75ecbbf67daf02e5314e4eeb12320800d80ad7638c91d6f1a65c76d734e6dae88fe3b28e95a32355996d3'),
 (30,'2d1ffb98dfb9d486747f160a2e73578a627c53f56cc73ca88353bac326ea6b6716e1e80e8a54980269727359f164315d'),
 (31,'c2c28a143b49f066b49a6195e6d1743868440494da06ccf59a9acb75fee8a6ec61b0256023e454854ecc249244cd8eae'),
 (32,'3d60ed94b6202318e01d03b05fda4da3a165f8ac5951034f609af6b40add04973a1fed178aa95b6afd48de261244af5a'),
 (33,'af0e888da79d2f59b6092990ef68bad5e2f561e6ebb7fceaec25be1145620be17cd0f45683d7972f3c56c3c52d08efbd'),
 (34,'922b56030d6efaf35d6a045e3420c9dde8b30185f604c1c9abd93819486ccf909d082d773d9ca7611b7e3d4a5197e3af'),
 (35,'c23ab31521af0493b15f3d5a8d7788d294e8bd8523da860c78f829ad5f94382bc27150d3e79a02a1ee797b9d6559ac4c'),
 (36,'130788c966d5f4dc844300ff4e7fcde9abc066337315ab4934d5323ac3fa3e62a8db823b422c7ac82e6291bac4b4506e'),
 (37,'d2c69725c11f50fcb7e9cf48331b00a2d5ef85e6e5828f381d004275bce286eec40142bd25874f9a18891ff7f9168ba6'),
 (38,'c1d4dc26af31ea053f04694c8d90565a7265f5185d100807be935644f7e39bfdcfb73564e10cb784d0fa8f3a18568dba'),
 (39,'6f964b1a786bf483fdf3d278304c8162579f66390129a77b1e91a0f9ed014384e16c2a353f6eab2a75704731e5325949'),
 (40,'5c48ed8e6296349d7a818045342b47001ced8d5780be040dbe08e5d98a359c98ab6e214520a841ad247737f36e4b613c'),
 (41,'a0d8622ea4889b69238b89501a14f2f3dad3fb8c6a1f7bb157fd554a3003aea541cea67e573875a94552eba7ace77b5c'),
 (42,'7ebdbefbb132716ad6918f45b59379416134071af1613b230b3b27f50625730e52593f80539a6f3528601dcecc855d3d'),
 (43,'8309eca6bb6f85fb56c50b11bcabfbec53d1c301e2037f22e7806534b947758278545edbe03cb91b754064e066e59864'),
 (44,'128e14ff338d461b9c460d69bf42d9dd74832bb66e45ac93a093cff47fc47761a144e3bedc790b9916f996b11a470ac8'),
 (45,'aa5dccf7098f2191e241dbf58855516350e794d410aca7d3c5894e6e523c346afbb487f7ba55074160948dc87388227b'),
 (46,'cc63f87020e2fa7d897bd6d4ebeddb9c29301d3a8785608aa84f57ba658636b9950a2a7570dbd7972a81cdfbfbe76d65'),
 (47,'c203f0dd5d26d61e3cadac84a85839a0a5705a68f121a4ab699f890949eac4c45a3c9adccb7149319332438966b170ad'),
 (48,'ed500a4b42b2ece253133fb969c1c8cadadbe4a12d33b3b11919acadfa640b43a1ee78342bc4a3f7c2203baf1a9dcf16'),
 (49,'ce431e273e6b0ddcc29fb8ce85e5faec8f1788462c507da2671e165aca8d169e23fcf3c90228dcf2b02888de259984a7'),
 (50,'3ac1d0105e675fc9818a2aa62819e48f6dda1f60d28aa1e366e95c43a9d80bce9218bb20bcb964a2c89eb291838c5cde'),
 (51,'da2b69cf3bb9ad3cde6abd210ee4fb38ad20a07d367bf30f4090d2843e21c5661f6d8b0a29da5aefa5ef61b329b9ebae'),
 (52,'dd3a7e28493e590a7f7dfeb8e5f2ad2c055914752678dc9d0403332ec87ec2e04147c1cf41cc752266ca20b7409ba5b1'),
 (53,'95416edc9552f7a2a75b82df8cf56209d14cc99e65b44b087f9c1fa7b0bd08be7741084224d696dfa7a562e12fb0e8ab'),
 (54,'6cc66d2baf1abc03e0de50b942df746fc67cf93bba3e1cdecb334843b253b5823a514459fa7e50aebf59c21813d742b4'),
 (55,'f729720690f3f1fcd6c74ccb3d47577ae464c2f8547a24e3f893cf0ef4a517849adf42631092fe1ce5818b9c7ec10a3a'),
 (56,'bff90795e98202b2a85b15bad55f29f321123f1819ee9ab76b927ed215257bf0c3bb338cbb9c234eaa3c697edee69ac8'),
 (57,'7fa52e9563ebde699a1111801db42c2dd15b10b7002b9ec6d18f3dd9adace5657db4610683c6253c62824ef4a55bf173'),
 (58,'fc2cba6a8225e9f20d7544378d53467670c339858198d8021f9cfc4d51ca0074c8e610894854b48076577583ba840097'),
 (59,'41417298452d185d2c89fa1f693c3f35fbc77295475a2e29389b352eff31996de27382a8a3a23530056dd1c5a6306928'),
 (60,'11fbf43e150c5ae0bc3f91c10d1ea2c3d110d586ad956f32a77a23f9c10293066efb125988e0f76d0bb09247b11a632c'),
 (61,'f12dfa736bbdb67b5b98f394109cc02f41a4d006ed1852e36e079feca60e13538d2f6aecb4e9f25984011a6e35da6cea'),
 (62,'cc76fa13769e8d10fd504b28bd7abf9fbbecf4c7d129c5cd165c837cea3b35735c105954ed3809e790beea5ea35593d9'),
 (63,'34b38ebc31460874b3342b9f9ffd0fdeb46813075b547d5d6e9dd9665402ebf691e412639a67a4f86c8fa308daa5b931'),
 (64,'e7103b2db48a971f7b8fcf49b5f6676ce6a2cbbd1e70e7e0c1e06324d2e6cfdc22c400be283e2d737a4515ac5b1b3903'),
 (65,'26687e7e3f9cc7e1079033470e48a1f6136665724b9e81c2facf01e968e0e62e5a41bfb687ebb3736325d3fb309a8ef8'),
 (66,'f72e67e6bd8bd9e41e26aa1b94a97e8d41099c9d66ecfc8f888d5180596aa808ef608c29ccf972755cea551f196ba187'),
 (67,'a049002589b2df5c3aff978c1c2d0b16c791193b512a0cb8ca4aa2eb50b82b65ce6b8daad6db4e4c6ec6ef48113ffa61'),
 (68,'6a5d5e141b07519cbae4eeac15479a1d657bb70b6eabdc886cc29b1b9fca536176eb1b232bfb438daa289939f8d8d2f0'),
 (69,'9984b8f83f6e012fcba1eb9227c73c428023ceca1c9bbe76700893b63bf9e438f277935b8b8eb9929cbfa4cffd0766b7'),
 (70,'72f25fe770b8fb3bf99cf600f8775365cd2b2ff5a57ecc57545837b05f5465a5fe6318dc0d37eb595990bbff1131bdbb'),
 (71,'ed6906ccbb2f177b7c6ee102e5b9ad896c952f635133f69b3c6e8484381d7d06710197012dc9ca646f62ac656be69313'),
 (72,'61d85a437721934ae16988ff103c696b4ea144e9415735139a7732ae005bf94f3bf2735139df6d3c2492f6eaea02c800'),
 (73,'514f3d73f604bb9bc6cc8961f9384b0ebb2eac51822a59897062b3acdda15011abd003585d8368865d64849a9a251c1a'),
 (74,'eaa12d5a5692fad09614b744cfbe031894d0921a68fb0aa89b5645948ac430767fdb3147bcbf8138400c2a9f7e338782'),
 (75,'9ec0263ad251db8d8f7819bfb028e272dfc7f7975d7b66d247ae3285325f6ae790418933267169dbce625c55d7d10d23'),
 (76,'84c9a1e613e5fd3a1a5ea613e997aeb2bc7771ff14efa63dbf238a8bfa0ab52127f73b1834fc49e95b1772dd968f2547'),
 (77,'d9e34622276b1e33fab09a2b0e2984585470b5a85ee5628fa2eca99d783d119cdee53bebd8ef2a565236a787c69065d3'),
 (78,'3c4109f418a7673e36f37935322430361ad21b72d5c85dfd2cbfa6b1d8a131ab699eb83bdd957973a3d30b3c3c277942'),
 (79,'ca6dd7dbaf31fb4e4f547660cf97ff08dac527907300f6d95c5d0207f0c03c761f9faa236f18b0f38f7300d3efd14c01'),
 (80,'28f736f00774fbe801dcba751164bac883d565e61040a55c0da3fc0ba05c382f189a1c3d20ceda41a6b17cc9f0207004'),
 (81,'687409fdb5576a6cbad763d841e66390fa4db82e28a756e54333abc3c90fac496a1df75f06281a66674937f034a69afa'),
 (82,'dcd1ea86570b5ef0bde9a705dae04962be8546837990df5aeb9eb81aa8f8810064f97bc0218474f7edd52e9ed7aa0036'),
 (83,'07d03dfdb5937cc6832f77e6b08b2083ce7e78cb278f48f8f764258fbd67b9d25dfae8131c5a88e55926a69b8341a45b'),
 (84,'df619f2e351ecb478079a8bf258f8a9a9f8695f13c63a3c8b06c7f24c280be2eba338113fd4988c33f864300cc224415'),
 (85,'d44c2fdc738c1b9165f603a1946ea5248bff1087625798697f5ab769f8933efe9291230ce553e9baead903351aba0662'),
 (86,'50fd4717398ed96ce6f942195240dd3d570167910c46a606d6284ded85740a84fc688e84c8caedca839ceb0ccd344820'),
 (87,'b0b787ca697d56080790f6dc79888dd600c5460d813dedac9c0accd480c348c4351ddcd5e3a0d7056613d7867392ab45'),
 (88,'6d4e843a4175afff9b0ddc141c10da20dc84051ecc81007e977d17e5b34fba54607a5450628ad98ab3531c0b41ac8392'),
 (89,'bb5cdfdcd60d2fbf3419b40bbc5faf2941044bb77b6d9ac4f0429a0ec86a047acde893d7a67a089a1807fd6675d3f8cc'),
 (90,'2358c18738aba8535b35f36b66c625b67ee581b682fbe09c83e0f5d41ffb540d7fe4cdbec3271d5b5b75428145402d7c'),
 (91,'e353a91a58d7798a9fd0b1f9c5d78f8392290c23c6702699b469c0ce6882ff1dab502fbcca37dfcc4b7af454e2f78ede'),
 (92,'8c72f7acdc95535ee14d8d20031c5dda5fb9e8b10ed6a849d632e43c88b9119320568c769f295b49f6c2d42e0a41a783'),
 (93,'25a6e38fa174f8afa91da34c8aa2799bb98e028b504f1d67b74fdd12814b2046fa9f0f1b2f51238dffda8e11ca2e9b18'),
 (94,'c1f23732db4d7287cd477d95fc5d6a39ea5dac2339063f3d468df0f44e3aba8753013f2b5663156e475820d47eae5d21'),
 (95,'0a0aee53c52f05699d5a66f9e83be3ad087067379d1ce64d157ae6cd28bd4ff9158c73caec68e8b30e5ca978a393e9e3'),
 (96,'1691062d3f7045be80b93dfadce4b82a40f2f7afbc926d24b22f6c5ac4cfbe566b4265cc5835a0c5191a30dcb7e674f1'),
 (97,'ba786150ff5f93fe98a88fc33d355598cb55cdb3814958cf0075a558a94f0c37bc9c75332b4b991ac09aa8bb8bb0eab5'),
 (98,'63aa28abd0fa597f7d3947384071edb2619d5c7b24f3796ce066042f91c06f88ebf361cdfbca7ff6ad08e7151e94b31d'),
 (99,'c1eaa590646b6ddccc38ab93b6cb643cb854bf29e8adc7435c8358a1da642d0190a741a67fac0d330a9404672a3299c1'),
 (100,'0edb07ded15bdd526df21be47666a1f4ad49bf96b9b250164558e830447aa4a3375290cf1d8ba924201e51cb984c5fc6'),
 (101,'060c821dbd72fef7eec2205f0969b8b83279a9de7b9b1e0fdb80ad49b1f1aa8566fab36470c8afc49095a1cdacadd2c1'),
 (102,'11504766954246ab46d0d70d68d17aa05714d5fa8a8d605ab8b022bd352b87ecd4c6daf668acb63f1defa09090e98c43'),
 (103,'d8db143aaa927e9e587244dfc480fe9e7956a7e93364884728dffb9abb7c15fd27fae78f01de8d32af0f49eaf1a6a3c6'),
 (104,'45c18f72ff37f26f465ec172d2ec3f060f9610e4a44eec85e27febf66e6a5d95201318cf58dfff0584883d12107b592f'),
 (105,'86fe155ba98451f2da9b39ded53dac1c4e74e290f4332f8ae01c69d40ff06278d7f3adbbedd3d9ea3cb1ea5314ef6c5a'),
 (106,'9046662be807be32ba9dc9a85ac51983fd44da659b3a80cc2d30a722ca00b5f2b726fa1686fcdca8b52bb80af841746c'),
 (107,'5e953318b3f6927e8164ba021addfd29e3f3eeea15ba984e582a90f4f3618362e4b94865c019d7d833737b574f472cf2'),
 (108,'8db7c0561a22d5319382b9debc193b57601f51b3567025c652d18ca40f1c2d8a5aad2ce7c55f15e8058a5b52a10db014'),
 (109,'160b1db4d1448c35cef46f0d7baceda735f021e61a4dbc575970712fefa19e31656d9a8eb7f4bc4e8af72480aeb89222'),
 (110,'8afeae0471167a5e50d6e165eda503423eb3ee8bd7bded4f4fdd3c79522f3e97024402f3f1d76f6dd0389124414de37c'),
 (111,'4cfdd074aa7db9e1635c039cb3f18a408ab86b9c512f8adc1954ef5ee7dc41faf3e3970f5a5eb9d60b086c23b9eebd6f'),
 (112,'e3711ebb243c5fb482cbeffcf4bb940f0d79c640d349147de34592930b6a183d3584ec4331da1264ac6c68eb5bf89bd4'),
 (113,'680c05b91798654ad570b650586e7c8eee41d7e8c8d02e3060e633a93ff651baf08030f224e35251d297a454f030aa6d'),
 (114,'cf245d1ba5ca7ab50a4d2d66ff61a04491fae691e0e347f3b73f59cefcfeda4373463581ec2602ad7eb09d4821cce0f7'),
 (115,'acdeb9bd8a7e51749c7bc7f025cfa484f6435127b62ae58c7d84a31122e3f94e6368d2a809ab25acd2e2874cd0d9577d'),
 (116,'74650d0f824239a1df65ca6ea36bac83f326695683fa3f764f5613d6f63e4f565726fe3657bdf3c367534885c7fe8768'),
 (117,'95da2eb1a58040f1691cc4c332b7852140dc7953d59aff10ce3a91886009b8eeb16573a7e79ec660c73755005be02af8'),
 (118,'4a32217cbb2fa56ce7e9d630ef2f3c9981fee5c2a2c7ba7f5ca1a6562cd7f8a45f10d8e8823f2dc325d95db28430c7c6'),
 (119,'0c9a590b1c2cfc3e72285a33c885c44a53ad8e6c36042b1198f194377227272b5d4faf7ebfa7fa71865bc8fc1babea5e'),
 (120,'16eff9604df617accb4357d25d235f4129a67c0a35bcb906d975ca51559ca273c54cfe61dfd6007e1b95c9e973f295c6'),
 (121,'19c3078e6a2bb1dfd1851d6309fdc4039919652e5c47757905c586a712a1753a4134e23229b7f6aa7848ad5c8320f511'),
 (122,'bf9f7b0fad7d7992c227d25b4f74457b58cbd546fed3a9d9e28786c19f381d1e8d194f688f978d7717f1e918915c84b5'),
 (123,'518f7122edbdfbaf717a070805f4581e1dbba85ef440de8ab34d537893d77de9331ca6816bbabf1ea67a4c4f4c0cc4e0'),
 (124,'6ed48f666b2ecaa7464caac3e13256448081f3e0b0176b101fec1a938e4f5708bd1936484809a42821d92dc455e47813'),
 (125,'8aa0b5f9a3464da1f7a0b58b80f9961724c68e96421ad2d896da940adf2cf2d95053da0ed42d78dd7b99e9a86b734196'),
 (126,'8709abec78817f8923850f3ef9c10f5248ed288227292a727d288190e110fce6a053b50c44e3ab67960cc171fba155c6'),
 (127,'a32d11a9538d9c7d5c4ae977b7199debfb0ed891081b9e91b34e6f8b86657f2c17c32da7f19d8ee5db649fb4cd866888'),
 (128,'53530028dffb8ef5f54df23db35f338f806ec7969f3222f9d403e540874c6b2f5e8b331d3ffff240ef94eb3f0d99e00d'),
 (129,'6e8650760be23bd5a604aed1bfee4e05b4f9e6e1fff076ec6c4bca1890c227129139b0848459c3f76c45c26af1cfae79'),
 (130,'e3b92a4197dcabb7196c8a2da673023c45552929f26ca7b8a97f6e32546b230f59fae9ea8cafdd9fe0b61094674bb0f7'),
 (131,'505378bbfbe858d216adea7680d3fd719e4e038a6c63f92217dc65aa4c3b83a07eaf56a71b4a6cc8a01889e26b1edc33'),
 (132,'0baf95c1028c5bc3c9d65093b3b1c44c37878004e500b626d2520be344093ad918f929b991c9371de3be55dd6dc27cdd'),
 (133,'59a616a449270e4ba589fb082e9b24887a194fbf2afcdcdb398086552bc24db8908dc3d603468f5b36ce43647f20ff33'),
 (134,'0c9112c2e127e0941f9cb1879d28affb753fa174ddd31489aa350975ef4a038247de75fb747a30537e8d5a12edc7d752'),
 (135,'bf66839b1cd85bda3903f5f157ea1485ed96035f86b347381ba75b91dcab2f181443d903793f620648764ff47f09adbd'),
 (136,'f31df5525c33e814d182d07233fcb04e712e508bca779243e3fd96fd878105f86cc56b563062a28e3fd4ae36c8c84022'),
 (137,'286cd33b30945dbf918750fa070306e2c59b0726e4d34652857c84731bf77b4f3d95aa64fecdf6155286c154a5975b63'),
 (138,'4bb209f6d2122075778e55a3a4821756e047783e1505f96463764775e6a41651a6ca70d46e6e5ce3e4a15fb13ac4981d'),
 (139,'1d65325c1e3c952635858624a0e4c25af8b242a140ebd0094bf0a837d2ef99676ea45bf098ca67ff65ed6a0c8e4477d7'),
 (140,'d68107b087ff37c1c0ca82fff7470c2078f0aaba5df4b3e5b0c74a884a0a2cb084d8e17aba01eebb0506269084f0cc08'),
 (141,'d54ab02ac2b87b957b7b950d9392206df319ab45f1a996a3ba30ac6497f0cc0945e56d6fc51e4e9bf39bc1ed64863158'),
 (142,'a08d851fa096616987e2d8c16f2fd8f8e30eaf63777750739069395f54a43c860c77346bbfd1965652149fde0d898e5c'),
 (143,'b4f6cb9064edcf28b34b11f0a52f87359ba9bc1e55b6d114d5172dea4851692f8bb7f5f8e46c3e68b7ac78afd25eb0e0'),
 (144,'b3b0e4a241d8bbe72baf914c25648f4286aa3e4d6e93b28d49cdd705d16492a73ac3550511ad5a1cca26b828bad9119e'),
 (145,'bbe9612542b3af873ad0be48585f6d8ee721e442c91fc755a62215d4f9ae94ef9742d45a0b8709a4b66c38e64222ddda'),
 (146,'e9bb5a02368a1a9471b2309e0bd0fb41b2193a7f53c51c941e71bc51dfa0fd0331d5ba820924548bdfb207143bd91686'),
 (147,'1430b4e855fa35898abed6461637bfea3cdadc3e1f9eac160d28dda9e82cdb327927e6887566da430c5ea55e6acf61f4'),
 (148,'7cd80443e400b82b5196aa6779976bdf7271ea55c651086d8592f4b5fc47e21e744a3efa3f258585d11cd08f524490bc'),
 (149,'72b9f1d0ae6bc5177c2421fd2aa3317d8895f8a667475d373fa0c4baae09525379376d4d7acd615bd29206b4c3fc17ff'),
 (150,'3dc30c7a860a66f789fa046ccc5d4ed7120cdd6312368cc785dad8ab10d94e7bf7f2ff7e4020a8ca5ab70aa946fdae22'),
 (151,'b852bacf19c3efcf707430614a29dc5f5140de84d8009db4305c1e1373ad8b52dc4e5a43071fddb840b2ffd94420f497'),
 (152,'e908aa1fac7e254aeea8024046c180408333643ba62931ddb25428f46abd33d8937d7f2176115558a3023b41b7466c9e'),
 (153,'cf94ab91fe29ea304cc994b2a5a176a62b65bc61d2aa49d0213163d9519bd19c4d6806f236353568f0976ca541b0e722'),
 (154,'d0c2cac03824de68b11290d8bf0425740e11aa17541156de4098f0d1ec747bfd146e09d3e312fffd12d6281776b2c68e'),
 (155,'a7864f1f67f35028f069cf3316580a29000e1383081c6d01895dca15df085dcb2bcb115afff5ed4b9162a4ccbcc6eba2'),
 (156,'fe8f2f890b0ad5a5a8d51deb62136b06d41d97575cbd88e8b615f5121222631930c0392258f73735ac2c55bbfd4b6838'),
 (157,'90281ee5098c997a38c26b14b31883e442dcfe5a8328b86f698d886ae8a401c40635430eef53a6d5000f95f6925d447b'),
 (158,'a6ff65b3c5caa1e8918bd8cf85fab744bfe57d99c3e08d652f8044b36200476c83f86561adfe6d69b49d68a46cef974b'),
 (159,'3abf97cf7ac868893382ed2f7d4b2610dae486409b3394dc4212cc30bc7971b16dea8f77f113e04b57b660c3d67fa400'),
 (160,'1c2478d1f50f4d69a148bb8bcde17a137e652784dfc5376ec76d88bcd8a8dfaae7af4424e2e1c89e7b05e935bfb57fac'),
 (161,'e8135e74041a52b5fbeefc2fd05daa97b56ac4ead46a6b5414a167fd662a4372a5be6c2ce4f73f1330bd15f62c016903'),
 (162,'6194bc62cfa25da1dd5dbcde7f66935c915c5316410ebc44db03619d94e34d11b1680e5a40c1e4de20684a3dc42e302e'),
 (163,'1935b1c285de4272b9411bf573361fb69842353f633f4192fe6e50aa5ba54defafdfe3403e828a20a307b35684907e48'),
 (164,'e2eaf80d402d9830adae0daef03de59cf4d7af12952b447edeba8b9d4bb09c42c418869643cc0a453df4e2d8b94b6660'),
 (165,'87cc0c80998192f65c4fc93a1705f252fbe62126e32e314e9f49250a4e02853998bc113a7c2c8224ebcea828a0b1597a'),
 (166,'0fb85388a8449229eaf408af11d3f60b34686ec95f049afe210563d98ff6294801889560c3f3c896768ec9227e834242'),
 (167,'095525e24340d56e14b19fee28a7613da13db7439f409dd7e364fdb6d0b798e48982adff9e3bcdd392c861a2bf78033b'),
 (168,'975abd1ad8583d0c2a8ef4209bb0e0e41e84cb020e7b6688d17b187f02d20584052cf7e078e2048f86033c1ad06ea54b'),
 (169,'495bc0142d574687e5d030694d1ecde0d5c39234a32e4ccf631c1bda854ccf02cfae720054915587f44de5dcf719f372'),
 (170,'8f2a88f3a2b21e87a4411350fcd3cb3dfecc831a20777695c9b101657a8fe9f162220cf893dd80c181bd506bc7df8187'),
 (171,'2b6dfc4b21426169494fb47c707264921aa23e4665ca02d0b67ee23d2d21a032bdb973c97343478c45a5ae1210b09958'),
 (172,'e58b84f06ad6b70c33275b30bfd71d4901db50852f4e18200d233bfeb91ae71b450d46c3f98ed60f91cb72c54b90db98'),
 (173,'a34294e2fd9748be13ae38fbe3b6e654855047978434f7a96c8a6d0b6feddd6e68400bb342be1d6b605f894d34feba87'),
 (174,'a7cd3b6285c776aa80c2d2ca3a782d4756f4cfe2f510ca82456b458047366ce3682bd86f0dee2871fe60a298acafb8a3'),
 (175,'020dec31fd171507f6e0a1f4aa6c11f79c6b5c05b574147bd66a23b17bbc8646c44175bc19fd750825e7a9590c9dc857'),
 (176,'dfb9a692eb09529e9a46377e93db9334a2ca479e79ef5465ea027475f707824c5540fd8de3a12b6069ea7adb8f5a1c0d'),
 (177,'8c2c752d556be30c95d7814230b3228d3b06ab4602bec5e3ca5445dec9e14c336fc9d9c354e2bc20a98e9040d96262b2'),
 (178,'87e1bb9723711332561b39b465fd0a8ae63187a3c987a055ed9818648aca356d4e2ad78f38052f59810680b3d9f038f5'),
 (179,'0ca74170689590871528b513cb26cc23f004faf06011f26b4acff1c023610590ec8e260d6d792c3547ee17af63e7be47'),
 (180,'e8a92bd717910cf122c9afa8ffb2bfa0e4c447415ef9c9aff6421f1592e9ddaef85699bf8f4e8cf046e85e5423d07971'),
 (181,'05b58ce9a18af92baf88361d82ae3249f8cf7d2e6c445b28f475e81c793ab5d3e273eda853de4fed827f58185c2ebb58'),
 (182,'6b161e747cfa1ad202b0e1da6e543bc164fc2bec812ca15ffb8d65d10cf44913d8051df30f8aa980ca916aa1296956ee'),
 (183,'fa3626d35b48b995f3b1e90a8524d4f80cd23ba1810bbdbaec1da6b8818a6f4f436a9dfd155f06fd701ef2fef01b0f70'),
 (184,'740e24bb50ac61cbf0f81d6966885845c72920a67c93b5bf62103a7387adee0902d71e0c4cf3a545dabea4abb245123c'),
 (185,'eed0515b89a55f717ded8d298cedaa398e45758ae8b7f007aea3839754e4a87e581de7290b84881c0d9afce979a291da'),
 (186,'ac99f53e6506757002576eeb1d0ac20fa3495b25e57d582e8b24ba72b3feaabd66937d13e6baebbc82d501eed69fd913'),
 (187,'51b185bc4de388c11e449e9e22f6730e36255bac2c706c47804047ce27378acf979b5be711d6d35e38d8214e23d18d24'),
 (188,'fe0719dc44e3f88409e6f5a691978049c6f240d5980cd97e1df9a56cbdf321b57a2849d36b9cb5ce215839af91e08aa4'),
 (189,'2e97dbe2165a41a8be7fae6a1a143c9ec99e12b272ccc42262763d16accc278dd635b4bdfbf66f9c683f000a42ac3730'),
 (190,'a9b31e217401d66c035357a2ae7e06055ca63d3e7d1c3fccc406785c5850b59954b8da5c87b3ebba286ea56c2fe5379b'),
 (191,'821366ffbabb98340336552e9a1eae16331ebface44f2af274eab44e5c2dbed5449b5a7a2e7bac6138dc24f75043bf0a'),
 (192,'c144b5c1b705a13195ecdbaa7a043f6d033254f95b89284138be156cfd41c3568045220266d08188b2e2d55e568fba99'),
 (193,'c6442171e050ff3418553f70121941ae56a6042fc69bdee9cdfbeb150bfaa3087cc6d314f3c9d5fbb6c33cb9a84d811d'),
 (194,'6636777e8f5bf69a444287536b254f316fb000c9a7bd2075fea83b9cd1f3951360783748878903d3457c699d71057b61'),
 (195,'8080263264abd05de7f2fe9253a2df76efa47690e24676f28603462bfc436f25c446959ab2f85bdc946e7615bd576d61'),
 (196,'6dd0674ca2f11656b1a04d3f6bd47e52713fc03a1345a3b37ed530995c60cf8669de08f75ad6801069be25c39060d402'),
 (197,'3d01ba57771ebdbbf86e2c5febbdb22a9844b9c3dfb9bf45e0b8c4adbf04dad347ce37d1d668a045939999a277190b1f'),
 (198,'2d75d5004d48d5d759b1a2fb6a62824d490139fdd819f57697fc07616d7e8691ff63a1218f61f058bede0112b397aa0b'),
 (199,'5d71359453e86f74cf193b4a9e36002a1f168266aac99f3fce81472e1df394022d04250b20b54dcc90bdff20a3673ab3'),
 (200,'88926bdb190d3fe1203b0876539f0bbb095a0db6b7fd99fafcfff94c578383c90f80839ff8acc563f8abafe330ce5944'),
 (201,'3460f07bbe894727aca489a228e14f68a3808bce67488d511d462151d31c14e2199d19a62425a41059112b1f755797cd'),
 (202,'44188d68d661f6c54cbe879d7c92ad5daa5e4b5296ae8b603cdde7b6e21d7dd3bddadcb5d5fd4b176c6f73a10afff9e4'),
 (203,'c90fbfdca89246a5f4176ec80ca81eb81cdd4279c9a0e6f2c2a225e064b969620c8516803e3649e68615e27c98b63d7c'),
 (204,'373fbf4498192612acf80bcf5b297fc4554218c8e798c197c47740dcf0284b49b4a674685691c52f81a5689a0a4219f7'),
 (205,'c385adffd077ca58fbf7e594efcf29a556b9d06126b093178b738f67d759ec8f663802339d3c54078deec8aa091607ed'),
 (206,'9564e5fbec6ca945d9e588dd6c397a456ddfe0f5ea0d21040779d9b00a4edf4c1935ed7e371df33570f498f9302508e0'),
 (207,'f316667b1bc7d5ee271f325a7ca05dacc00c81c01026e0f1e50f805813e61ed76515746097e77ac45156907a503553e3'),
 (208,'67aea8e55a50b144bd212ec86fc395757edbd56a775f53bdd96da781616f6fe85c580dc4470d0965154d92e4912fa744'),
 (209,'2e3579ad7996b38b11dc67f628f5b2e28b4498d6f7d91c794a577f2992d1a106769f4b3e49b25ce0bc7e01d526816cd2'),
 (210,'437e213d08b274a6750c5c5c0a1c47db0915608ed1be57595850050128632c6a2922ea8bbf3c51f296271ad11521cb04'),
 (211,'2a4762c24a876378086ed791de328ba6be9d14b73c6da819f1988ce59a440f6a91f29f4dcc25e7ce8d1bf3e6096c562a'),
 (212,'e39a49097ef8eaf3cd299c90c968cf9373dec621cafc06fb5570fccea3db8af940d87e813169ca37b1af7bb0504b42e0'),
 (213,'96a735bedfcac3efb6543bc34a53f65b151f017507ef15fa4b8b52d82cd158c650cb50e1d3d0e0ecae444aaed564cccf'),
 (214,'465efb6550e8bd62083935ed5c75b39a140a1cfb151a4adf5fea3ac03eafc07eb238f137ac77c1ad8c4a8e91df56bdf5'),
 (215,'da27244d17d0e3d2c864148770e32f4e35fff815c67dbbf2ac18ef89d0f644c700b779677232b75149e7de261afc252e'),
 (216,'ed95fd6de7c6de897b8a6b9fb1c93aa320fc23d86184de5669142dda8a7ebe1e7ca4fcf1fb67ae7e79ff7b6e308ceb7f'),
 (217,'74120cc5319976843440b25208c448f9da28857cb309844efa6de29ffce734d6c8e762ab1538ea879554cd17d48436b3'),
 (218,'53c4dfeb76cade36b76bc36db6ae261d7148b886e484293f18737e8af803c359d5cfeffff50d68fa8909875aea8dfc2b'),
 (219,'75905740323952bd207fb0bb6822afbf365b3fc3370e877c5a305f87ec8ddc525bcf257c7318ad3b8bf2e5152d830233'),
 (220,'2ca34d16a157ec631d4ab599b77c27837a9241070ddbea3eac709ab45cf3ab298a99c84931924399d1cfa2fa070246ee'),
 (221,'2d9e65b180e8cee447684d132348f8a80193ec584687125b6e9a07de8cf0a5834a5fa67d302c9466651f30444a961852'),
 (222,'fd70d4ef27c1503829f61bf4d16e45a3f415ebcf561fabbced0135a5bf29868e3503dab652630a392d319192af13e891'),
 (223,'6a18b70032c039d930c6b051413efa247734d97b336443f99ba0ad354be431e1084122327ce055198296f2a4a5d56b6e'),
 (224,'c7993050fa899cb736ba8f179b9f2f0ba4dcbe8062401317fabd83232e507452d76990732046ffe8744acab035cfaf45'),
 (225,'5ac72758cfa1829a397d7e978e4bb0bdd666d0384fc5a9b10d38acef84cfa502b06d69fd5ebdba1cfb6f0216019f75fc'),
 (226,'983ef9cb798b0d15938080502a3aa09ef6f3461bb4ab686f02da484ce02e188d974efb4850578f73c9cbefe73fff9102'),
 (227,'af0cb575b06032a376082581c1b745ef11590629b8df761ad0eec4d949fcc854efaaf6724a990be8cb0b210ca50b0b37'),
 (228,'b02632c8d3a8cffb059055d54988ad68b19fb06045bbdf5b40650f8c12ab0434454d2b1d1488831a564072573ff0acf0'),
 (229,'08ece6d6edf558d06eff11297da18184e9a762ff4f5455fab135c84a1b34dbef449b6c957f3d7fce197af113f821a659'),
 (230,'69b9f0de4175868ce73f01ce52ac47ee5607ae6d88c91ab9bc519112ea1cb9682945c2cc0b87a91a78edb32a2b3fade1'),
 (231,'84ff6af6b0a607b596a7b9cfb126ed10c7ab744f6a3abbb005569a1fc182ec055afae72fb9c5b793bee376dec316fbe2')
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_mismatch'; END IF;
 IF (WITH required_relations(name) AS (
VALUES
 ('account_context_candidates'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('accounts'),
 ('audit_events'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('cedar_policy_catalog_entries'),
 ('company_actors'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('deployment_operator_head'),
 ('deployment_operator_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employee_person_bindings'),
 ('employees'),
 ('employment_revisions'),
 ('employment_source_bindings'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('leave_balance_import_receipts'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('native_company_property_refs'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_results_v1'),
 ('native_group_process_versions_v1'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('ont_action_command_receipts'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('org_units'),
 ('organizations'),
 ('person_revisions'),
 ('persons'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
 ) SELECT count(*)=83 AND count(DISTINCT c.oid)=83 AND count(DISTINCT n.oid)=1
    AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
        AND NOT c.relispartition AND n.nspname='public') IS TRUE
   FROM required_relations required
   LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 locked_relations:=0;
 FOR relation_name IN WITH required_relations(name) AS (
VALUES
 ('account_context_candidates'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('accounts'),
 ('audit_events'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('cedar_policy_catalog_entries'),
 ('company_actors'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('deployment_operator_head'),
 ('deployment_operator_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employee_person_bindings'),
 ('employees'),
 ('employment_revisions'),
 ('employment_source_bindings'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('leave_balance_import_receipts'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('native_company_property_refs'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_results_v1'),
 ('native_group_process_versions_v1'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('ont_action_command_receipts'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('org_units'),
 ('organizations'),
 ('person_revisions'),
 ('persons'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
 ) SELECT c.relname::text FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 IF locked_relations<>83 OR (WITH required_relations(name) AS (
VALUES
 ('account_context_candidates'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('accounts'),
 ('audit_events'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('cedar_policy_catalog_entries'),
 ('company_actors'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('deployment_operator_head'),
 ('deployment_operator_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employee_person_bindings'),
 ('employees'),
 ('employment_revisions'),
 ('employment_source_bindings'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('leave_balance_import_receipts'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('native_company_property_refs'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_results_v1'),
 ('native_group_process_versions_v1'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('ont_action_command_receipts'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('org_units'),
 ('organizations'),
 ('person_revisions'),
 ('persons'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
 ) SELECT count(*)=83 AND count(DISTINCT l.relation)=83
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.relation_locks_missing'; END IF;
 SELECT classified.state,classified.variant INTO observed_phase,variant_name FROM (
-- Generated read-only complete Group serving custody.
-- Absent namespaces allow historical verification, never arbitrary installation.
WITH full83 AS (
-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates'),
 ('deployment_operator_receipts'),
 ('deployment_operator_head'),
 ('audit_events'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('cedar_policy_catalog_entries'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_property_refs'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('organizations'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users'),
 ('org_units'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_results_v1'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_versions_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE (n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
 OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN ('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))
 OR (n.nspname,p.proname) IN (VALUES ('public','identity_native_group_process_execute_v1'),('public','identity_native_group_process_incarnation_selector_v1'),('public','identity_native_group_process_material_v1'),('public','identity_native_group_process_navigation_candidates_v1'),('public','identity_native_group_process_prepare_v1'),('public','native_group_process_accept_snapshot_v1'),('public','native_group_process_account_material_v1'),('public','native_group_process_action_roster_v1'),('public','native_group_process_assert_current_v1'),('public','native_group_process_assert_input_v1'),('public','native_group_process_assert_result_v1'),('public','native_group_process_audit_guard_v1'),('public','native_group_process_audit_truncate_guard_v1'),('public','native_group_process_classify_v1'),('public','native_group_process_command_guard_v1'),('public','native_group_process_complete_snapshot_v1'),('public','native_group_process_current_context_v1'),('public','native_group_process_decode_v1'),('public','native_group_process_deferred_guard_v1'),('public','native_group_process_effect_guard_v1'),('public','native_group_process_group_guard_v1'),('public','native_group_process_head_bytes_v1'),('public','native_group_process_head_ref_v1'),('public','native_group_process_head_statement_v1'),('public','native_group_process_i64_v1'),('public','native_group_process_immutable_statement_v1'),('public','native_group_process_input_guard_v1'),('public','native_group_process_micros_v1'),('public','native_group_process_original_material_v1'),('public','native_group_process_participant_guard_v1'),('public','native_group_process_policy_head_bytes_v1'),('public','native_group_process_policy_ref_v1'),('public','native_group_process_registered_actions_v1'),('public','native_group_process_registration_bytes_v1'),('public','native_group_process_result_bytes_v2'),('public','native_group_process_source_v1'),('public','native_group_process_text_v1'),('public','native_group_process_topology_fence_v1'),('public','native_group_process_uuid_v1'),('public','native_group_process_validate_frame_v1'),('public','native_group_process_version_bytes_v1'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_session_shared_material_v1(uuid,uuid)'),
   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),
   ('public.account_company_setup_eligibility_v1(uuid)'),
   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=18 AND count(oid)=18 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=27 AND count(oid)=27 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=216 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=27 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), company_startup_rights AS (
 SELECT
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=83 AND count(oid)=83 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=92 AND count(oid)=92 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=736 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=83 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS (
 SELECT jsonb_build_object(
  'native_group_process_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE starts_with(c.relname,'native_group_process') OR starts_with(c.relname,'native_group_identity_policy') OR starts_with(c.relname,'identity_native_group_process')),
  'native_group_process_schema_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname) FROM pg_namespace n WHERE starts_with(n.nspname,'native_group_process') OR starts_with(n.nspname,'native_group_identity_policy') OR starts_with(n.nspname,'identity_native_group_process')),
  'native_group_process_type_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,t.typname,t.typtype,pg_get_userbyid(t.typowner)) ORDER BY n.nspname,t.typname) FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy') OR starts_with(t.typname,'identity_native_group_process')),
  'native_group_process_routine_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),p.prokind,pg_get_userbyid(p.proowner)) ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy') OR starts_with(p.proname,'identity_native_group_process')),
  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),
  'required_schemas',(SELECT jsonb_agg(jsonb_build_object(
    'name',required.name,'present',n.oid IS NOT NULL,
    'owner',CASE WHEN required.name='pg_catalog' AND n.nspowner=(SELECT proowner FROM deployment_builtin)
       AND owner.rolsuper THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',owner.rolname) END,
    'owner_superuser',owner.rolsuper,'acl_is_null',n.nspacl IS NULL,
    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
       CASE WHEN required.name='pg_catalog' AND a.grantor=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
       CASE WHEN a.grantee=0 THEN jsonb_build_array('public')
         WHEN required.name='pg_catalog' AND a.grantee=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
       a.privilege_type,a.is_grantable) ORDER BY
       CASE WHEN a.grantor=n.nspowner THEN '' ELSE pg_get_userbyid(a.grantor) END,
       CASE WHEN a.grantee=n.nspowner THEN '' WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable)
      FROM aclexplode(COALESCE(n.nspacl,acldefault('n',n.nspowner))) a),'[]'::jsonb),
    'effective_rights',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
       has_schema_privilege(r.oid,n.oid,'USAGE'),has_schema_privilege(r.oid,n.oid,'CREATE'),
       has_schema_privilege(r.oid,n.oid,'USAGE WITH GRANT OPTION'),
       has_schema_privilege(r.oid,n.oid,'CREATE WITH GRANT OPTION')) ORDER BY r.rolname)
      FROM (SELECT oid,rolname FROM protected_roles UNION SELECT oid,rolname FROM deployment_observer_active) r)
    ) ORDER BY required.name)
    FROM (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')) required(name)
    LEFT JOIN pg_namespace n ON n.nspname=required.name LEFT JOIN pg_roles owner ON owner.oid=n.nspowner),
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_group_process_startup_rights_valid FROM snapshots
), phase_pairs(variant,predecessor83,installed83) AS (
VALUES
 ('plain','ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862','3f5972d2e5c1d7277e71b4f716b79a405d152ca5bbab25b614dbc1fc67c5fd7f'),
 ('observer','cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6','304e176d646edf62e8767a9f986879abc74753a3b2a8812c1a8cec8447a20b78')
), matching_phase(variant,phase) AS (
 SELECT p.variant,'installed'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.installed83
  AND f.native_group_process_startup_rights_valid IS TRUE
 UNION ALL
 SELECT p.variant,'predecessor'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.predecessor83
  AND f.native_group_process_startup_rights_valid IS TRUE
), namespace_absence AS (
 SELECT snapshot->>'native_group_process_relation_namespace' IS NULL
  AND snapshot->>'native_group_process_schema_namespace' IS NULL
  AND snapshot->>'native_group_process_type_namespace' IS NULL
  AND snapshot->>'native_group_process_routine_namespace' IS NULL AS valid FROM full83
)
SELECT CASE
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='installed'
 THEN 'native_group_process_navigation.finalized'
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='predecessor'
 THEN 'native_group_process_navigation.head_revision_required'
 WHEN (SELECT count(*) FROM matching_phase)=0
  AND (SELECT valid FROM namespace_absence) IS TRUE
 THEN 'native_group_process_navigation.absent'
 ELSE 'native_group_process_navigation.profile_mismatch' END AS state,(SELECT variant FROM matching_phase) AS variant
) classified;
 IF (observed_phase IS DISTINCT FROM 'native_group_process_navigation.head_revision_required'
     AND observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized') OR variant_name IS NULL THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 expected_variant:=variant_name;
 IF observed_phase='native_group_process_navigation.head_revision_required' THEN
  EXECUTE $native_group_process_navigation_source$CREATE OR REPLACE FUNCTION public.identity_native_group_process_navigation_candidates_v1(
 p_actor uuid,p_family uuid,p_original bytea) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE original jsonb; snapshot_text text; captured jsonb; snapshot_bytes bytea;
 old_groups uuid[]:=ARRAY[]::uuid[]; current_groups uuid[]; guarded_groups uuid[]; after_groups uuid[];
 selected_group uuid; candidate jsonb; previous_group uuid; group_value jsonb; history_value jsonb;
 candidates jsonb:='[]'::jsonb; groups_value jsonb:='[]'::jsonb; consent_value jsonb:='[]'::jsonb;
 material jsonb; account_value jsonb; designation_value jsonb; item record; v_navigation_head_revision bigint;
 budget integer:=1048576; additional integer; first_group boolean:=true; first_history boolean;
 root public.accounts; security public.account_security; enrollment public.account_security_events;
 family record; terms record; designation public.deployment_operator_head; designation_receipt public.deployment_operator_receipts;
 rows_to_validate jsonb:='[]'::jsonb; checked_row jsonb; expected_keys text[];
 -- Exact schema types/nullability; no field-name inference. Native closure
 -- tightens nullable database columns only where that row purpose requires it.
 row_shapes jsonb:=$shapes${"account_security":{"account_id":["uuid",false],"context_generation":["integer",false],"revision":["integer",false],"security_generation":["integer",false],"security_state":["string",false],"updated_at":["timestamp",false]},"account_security_events":{"account_id":["uuid",false],"actor_account_id":["uuid",false],"evidence_ref":["object",false],"id":["uuid",false],"kind":["string",false],"occurred_at":["timestamp",false],"payload":["object",false],"session_id":["uuid",false]},"accounts":{"created_at":["timestamp",false],"id":["uuid",false]},"company_enrollment_receipts":{"account_id":["uuid",false],"action_refs":["array",false],"administrative_account_id":["uuid",false],"catalog_version":["string",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_at":["timestamp",false],"designation_receipt_id":["uuid",false],"group_id":["uuid",false],"input_digest":["bytea",false],"manifest_digest":["bytea",false],"org_id":["uuid",false],"property_refs":["array",false],"receipt_id":["uuid",false],"root_assignment_id":["uuid",false],"root_revision":["integer",false],"session_id":["uuid",false]},"company_enrollment_requests":{"account_id":["uuid",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_receipt_id":["uuid",false],"created_at":["timestamp",false],"designation_receipt_id":["uuid",false],"expires_at":["timestamp",false],"input_bytes":["bytea",true],"input_digest":["bytea",false],"state":["string",false],"terminal_at":["timestamp",false]},"consent":{"accepted_at":["timestamp",false],"content_sha256":["bytea",false],"manifest_sha256":["bytea",false],"terms_kind":["string",false]},"deployment_operator_head":{"account_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"receipt_id":["uuid",false],"revision":["integer",false],"singleton":["integer",false],"system_identifier":["string",false]},"deployment_operator_receipts":{"account_id":["uuid",false],"command_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"expected_revision":["integer",false],"expected_security_generation":["integer",true],"kind":["string",false],"reason":["string",true],"receipt_id":["uuid",false],"recorded_at":["timestamp",false],"revision":["integer",false],"system_identifier":["string",false]},"family":{"account_security_generation":["integer",false],"assurance":["string",false],"auth_time":["timestamp",false],"created_at":["timestamp",false],"org_id":["uuid",true],"protocol":["string",false],"revoked_at":["timestamp",true],"user_id":["uuid",false]},"group_authority_heads":{"group_id":["uuid",false],"incarnation":["uuid",false],"revision":["integer",false],"state":["string",false]},"groups":{"created_at":["timestamp",false],"id":["uuid",false],"name":["string",false],"origin_account_id":["uuid",false],"origin_command_id":["uuid",false],"origin_receipt_id":["uuid",false],"slug":["string",false],"status":["string",false],"updated_at":["timestamp",false]},"native_group_identity_policy_heads_v1":{"activated_at":["timestamp",false],"activation_receipt_id":["uuid",false],"codec_contract_digest":["bytea",false],"first_actor_account_id":["uuid",false],"first_command_id":["uuid",false],"first_input_digest":["bytea",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"revision":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"native_group_process_head_revisions_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_heads_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_versions_v1":{"account_possession_procedure":["string",false],"actor_account_id":["uuid",false],"admitted_at":["timestamp",false],"adopt_command_id":["uuid",false],"content_digest":["bytea",false],"designation_receipt_id":["uuid",false],"designation_revision":["integer",false],"duplicate_contradictory_claim_procedure":["string",false],"escalation_adjudication_procedure":["string",false],"evidence_minimization_retention_description":["string",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"input_digest":["bytea",false],"intended_claimant_matching_procedure":["string",false],"method":["string",false],"operator_responsibility":["integer",false],"physical_human_evidence_procedure":["string",false],"policy_head_digest":["bytea",false],"policy_revision":["integer",false],"process_id":["uuid",false],"qualification_criteria_instruction":["string",false],"recipient_responsibility":["string",false],"schema_id":["string",false],"title":["string",false],"version":["integer",false]},"source":{"cedar_language_version":["string",false],"cedar_sdk_version":["string",false],"codec_contract_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"terms_head":{"manifest_sha256":["bytea",false],"release_receipt_id":["uuid",false],"revision":["integer",false]}}$shapes$::jsonb;
 document jsonb; required text; incarnation uuid; row_key text; row_value jsonb;
 entry jsonb; revision_value text; field_shape jsonb; value_text text; parsed_time timestamptz;
 birth_request public.company_enrollment_requests; birth_receipt public.company_enrollment_receipts;
 birth_effect public.company_enrollment_effect_bindings; decoded_birth record;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR current_setting('server_encoding') IS DISTINCT FROM 'UTF8'
  OR num_nonnulls(p_actor,p_family)<>2
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_family) THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 IF p_original IS NOT NULL THEN
  IF octet_length(p_original) NOT BETWEEN 1 AND 1048576 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  BEGIN
   snapshot_text:=convert_from(p_original,'UTF8'); original:=snapshot_text::jsonb;
  EXCEPTION WHEN invalid_text_representation OR character_not_in_repertoire THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END;
  -- Equality rejects duplicate members and alternate serialization before any
  -- source selector is used. This is an internal observation, not permission.
  IF convert_to(original::text,'UTF8') IS DISTINCT FROM p_original
   OR jsonb_typeof(original) IS DISTINCT FROM 'object'
   OR original->>'protocol' IS DISTINCT FROM 'GROUP_PROCESS_NAVIGATION_SOURCE_V1'
   OR ARRAY(SELECT key FROM jsonb_object_keys(original) key ORDER BY key COLLATE "C")
      IS DISTINCT FROM ARRAY['account','designation','groups','protocol']
   OR original#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR original#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR jsonb_typeof(original->'groups') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(original->'groups')>256 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(original->'groups') LOOP
   IF jsonb_typeof(candidate) IS DISTINCT FROM 'object'
    OR jsonb_typeof(candidate->'group_id') IS DISTINCT FROM 'string'
    OR candidate->>'group_id' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    OR jsonb_typeof(candidate->'group_incarnation') IS DISTINCT FROM 'string'
    OR candidate->>'group_incarnation' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   selected_group:=(candidate->>'group_id')::uuid;
   incarnation:=(candidate->>'group_incarnation')::uuid;
   IF '00000000-0000-0000-0000-000000000000'::uuid IN(selected_group,incarnation)
    OR (previous_group IS NOT NULL AND selected_group<=previous_group) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   old_groups:=array_append(old_groups,selected_group); previous_group:=selected_group;
  END LOOP;
  document:=original;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   v_navigation_head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>v_navigation_head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    v_navigation_head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF v_navigation_head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 END IF;

 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO current_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(current_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL THEN
  SELECT coalesce(array_agg(id ORDER BY id),ARRAY[]::uuid[]) INTO guarded_groups
   FROM (SELECT DISTINCT unnest(old_groups||current_groups) id) required_guards;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM public.native_group_process_group_guard_v1(selected_group,false);
  END LOOP;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM 1 FROM public.group_authority_lock_shared_v1(selected_group);
  END LOOP;
  SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
   FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
  IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF NOT after_groups<@guarded_groups THEN
   RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
    'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
  END IF;
  current_groups:=after_groups;
 END IF;

 -- Enumeration takes no Group locks. Account guards end with this transaction
 -- before per-candidate Landing7 scopes. Final recheck has taken all Group
 -- guards already; neither branch acquires a new Group guard after Account.
 material:=public.native_group_process_account_material_v1(p_actor,p_family);
 SELECT a.* INTO STRICT root FROM public.accounts a WHERE a.id=p_actor;
 SELECT s.* INTO STRICT security FROM public.account_security s WHERE s.account_id=p_actor;
 SELECT e.* INTO STRICT enrollment FROM public.account_security_events e WHERE e.account_id=p_actor AND e.kind='ENROLLED';
 SELECT f.* INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_actor,p_family) f;
 SELECT h.* INTO STRICT terms FROM public.account_terms_registration_head_v1() h;
 FOR item IN SELECT c.* FROM public.account_login_consent_v1(p_actor) c ORDER BY c.terms_kind COLLATE "C" LIMIT 9 LOOP
  IF jsonb_array_length(consent_value)>=8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  additional:=octet_length(convert_to(to_jsonb(item)::text,'UTF8'))+2;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  consent_value:=consent_value||jsonb_build_array(to_jsonb(item)); budget:=budget-additional;
 END LOOP;
 IF jsonb_array_length(consent_value)=0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 account_value:=jsonb_build_object('actor_account_id',p_actor,'session_id',p_family,
  'root',to_jsonb(root),'security',to_jsonb(security),'family',to_jsonb(family),
  'enrollment',to_jsonb(enrollment),'consent',consent_value,'terms_head',to_jsonb(terms));
 PERFORM public.account_company_setup_eligibility_v1(p_actor);
 SELECT h.* INTO designation FROM public.deployment_operator_head h WHERE h.singleton=1 FOR SHARE OF h;
 IF FOUND THEN
  SELECT r.* INTO STRICT designation_receipt FROM public.deployment_operator_receipts r WHERE r.receipt_id=designation.receipt_id;
  IF (designation.receipt_id,designation.revision,designation.account_id,designation.system_identifier,designation.database_name,designation.database_oid)
   IS DISTINCT FROM (designation_receipt.receipt_id,designation_receipt.revision,designation_receipt.account_id,
    designation_receipt.system_identifier,designation_receipt.database_name,designation_receipt.database_oid)
   OR designation_receipt.expected_revision IS DISTINCT FROM designation_receipt.revision-1
   OR ((designation_receipt.kind='DESIGNATE' AND designation_receipt.revision=1
     AND designation_receipt.expected_security_generation>0 AND designation_receipt.reason IS NULL)
    OR (designation_receipt.kind='REVOKE' AND designation_receipt.revision>1
     AND designation_receipt.expected_security_generation IS NULL AND designation_receipt.reason IS NOT NULL
     AND octet_length(designation_receipt.reason) BETWEEN 1 AND 512
     AND length(btrim(designation_receipt.reason,E' \t\n\r\f\013'))>0)) IS DISTINCT FROM true
   OR designation.system_identifier IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR designation.database_name IS DISTINCT FROM current_database()::text
   OR designation.database_oid IS DISTINCT FROM (SELECT oid::bigint FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  designation_value:=jsonb_build_object('head',to_jsonb(designation),'receipt',to_jsonb(designation_receipt));
 END IF;
 captured:=jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_SOURCE_V1',
  'account',account_value,'designation',designation_value,'groups',groups_value);
 -- The base already contains consent. Reset to its exact canonical byte size;
 -- subsequent accounting replaces the empty array with bounded complete rows.
 budget:=1048576-octet_length(convert_to(captured::text,'UTF8'));
 IF budget<0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 FOREACH selected_group IN ARRAY current_groups LOOP
  -- One SPI snapshot binds policy/head/content. History is the immutable prefix
  -- ending at that captured head, even when enumeration races a later commit.
  SELECT jsonb_build_object('group_id',g.id,'group_incarnation',t.incarnation,
    'group',to_jsonb(g),'topology',to_jsonb(t),'birth_request',to_jsonb(q),'birth_receipt',to_jsonb(r),
    'birth_designation_receipt',to_jsonb(d),'source',public.native_group_process_source_v1(g.id,t.incarnation),
    'policy',to_jsonb(p),'head',to_jsonb(h),'version',to_jsonb(v),'history','[]'::jsonb)
   INTO STRICT group_value FROM public.groups g
   LEFT JOIN public.group_authority_heads t ON t.group_id=g.id
   LEFT JOIN public.company_enrollment_receipts r ON
    (r.account_id,r.command_id,r.receipt_id,r.group_id)=(g.origin_account_id,g.origin_command_id,g.origin_receipt_id,g.id)
   LEFT JOIN public.company_enrollment_requests q ON
    (q.account_id,q.command_id,q.committed_receipt_id,q.state)=(r.account_id,r.command_id,r.receipt_id,'COMMITTED')
   LEFT JOIN public.deployment_operator_receipts d ON d.receipt_id=r.designation_receipt_id
   LEFT JOIN public.native_group_identity_policy_heads_v1 p ON (p.group_id,p.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_heads_v1 h ON (h.group_id,h.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_versions_v1 v ON
    (v.group_id,v.group_incarnation,v.process_id,v.version)=(h.group_id,h.group_incarnation,h.process_id,h.content_version)
   WHERE g.id=selected_group;
  incarnation:=(group_value->>'group_incarnation')::uuid;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOREACH required IN ARRAY ARRAY['group','topology','birth_request','birth_receipt','birth_designation_receipt','source'] LOOP
   IF jsonb_typeof(group_value->required) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  END LOOP;
  IF group_value#>>'{birth_designation_receipt,kind}' IS DISTINCT FROM 'DESIGNATE'
   OR group_value#>>'{birth_designation_receipt,system_identifier}' IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR group_value#>>'{birth_designation_receipt,database_name}' IS DISTINCT FROM current_database()::text
   OR group_value#>>'{birth_designation_receipt,database_oid}' IS DISTINCT FROM (SELECT oid::bigint::text FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  SELECT q.* INTO STRICT birth_request FROM public.company_enrollment_requests q WHERE
   (q.account_id,q.command_id)=((group_value#>>'{group,origin_account_id}')::uuid,(group_value#>>'{group,origin_command_id}')::uuid);
  SELECT r.* INTO STRICT birth_receipt FROM public.company_enrollment_receipts r WHERE
   (r.account_id,r.command_id)=(birth_request.account_id,birth_request.command_id);
  SELECT b.* INTO STRICT birth_effect FROM public.company_enrollment_effect_bindings b WHERE
   (b.account_id,b.command_id)=(birth_request.account_id,birth_request.command_id);
  IF to_jsonb(birth_request) IS DISTINCT FROM group_value->'birth_request'
   OR to_jsonb(birth_receipt) IS DISTINCT FROM group_value->'birth_receipt'
   OR birth_request.state IS DISTINCT FROM 'COMMITTED'
   OR (birth_request.codec_version,birth_request.input_digest,birth_request.designation_receipt_id,
       birth_request.committed_receipt_id,birth_request.terminal_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.receipt_id,birth_receipt.committed_at)
   OR (birth_effect.codec_version,birth_effect.input_digest,birth_effect.designation_receipt_id,
       birth_effect.org_id,birth_effect.group_id,birth_effect.receipt_id,birth_effect.administrative_account_id,
       birth_effect.catalog_version,birth_effect.manifest_digest,birth_effect.session_id,birth_effect.started_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.org_id,birth_receipt.group_id,birth_receipt.receipt_id,birth_receipt.administrative_account_id,
       birth_receipt.catalog_version,birth_receipt.manifest_digest,birth_receipt.session_id,birth_receipt.committed_at)
   OR birth_receipt.codec_version<>1 OR birth_receipt.root_revision<>1
   OR birth_receipt.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
   OR birth_receipt.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')
   OR birth_receipt.committed_at<birth_request.created_at OR birth_receipt.committed_at>=birth_request.expires_at
   OR public.native_group_process_micros_v1(birth_receipt.committed_at) IS DISTINCT FROM
      public.native_group_process_micros_v1((group_value#>>'{group,created_at}')::timestamptz)
   OR group_value#>>'{group,created_at}' IS DISTINCT FROM group_value#>>'{group,updated_at}'
   OR birth_receipt.account_id::text IS DISTINCT FROM group_value#>>'{birth_designation_receipt,account_id}'
   OR group_value#>>'{birth_designation_receipt,revision}' IS DISTINCT FROM '1'
   OR group_value#>>'{birth_designation_receipt,expected_revision}' IS DISTINCT FROM '0'
   OR group_value#>>'{birth_designation_receipt,expected_security_generation}' IS NULL
   OR (group_value#>>'{birth_designation_receipt,expected_security_generation}')::bigint<1
   OR group_value#>'{birth_designation_receipt,reason}' IS DISTINCT FROM 'null'::jsonb
   OR (SELECT count(*) FROM public.company_enrollment_request_events e
      WHERE (e.account_id,e.command_id)=(birth_request.account_id,birth_request.command_id))<>2
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,1)
      AND e.from_state IS NULL AND e.to_state='PENDING' AND e.reason_code='PREPARED'
      AND e.occurred_at=birth_request.created_at AND e.actor_account_id=birth_request.account_id)
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,2)
      AND e.from_state='PENDING' AND e.to_state='COMMITTED' AND e.reason_code='COMMITTED'
      AND e.occurred_at=birth_receipt.committed_at AND e.actor_account_id=birth_receipt.account_id
      AND e.session_id=birth_receipt.session_id) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  -- Retention may remove terminal payload; decode only retained historical bytes.
  IF birth_request.input_bytes IS NOT NULL THEN
   SELECT * INTO STRICT decoded_birth FROM public.company_enrollment_decode_input_v1(birth_request.input_bytes);
   IF (decoded_birth.codec_version,decoded_birth.account_id,decoded_birth.command_id,decoded_birth.input_digest,
       decoded_birth.administrative_account_id,decoded_birth.company_name)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.account_id,birth_receipt.command_id,birth_receipt.input_digest,
       birth_receipt.administrative_account_id,group_value#>>'{group,name}')
    OR decoded_birth.group_id IS NOT NULL THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END IF;
  additional:=octet_length(convert_to(group_value::text,'UTF8'))+CASE WHEN first_group THEN 0 ELSE 2 END;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  budget:=budget-additional;
  history_value:='[]'::jsonb; first_history:=true;
  v_navigation_head_revision:=(group_value#>>'{head,head_revision}')::bigint;
  FOR item IN SELECT h.*,i.operation,i.input_bytes FROM public.native_group_process_head_revisions_v1 h
   LEFT JOIN public.native_group_process_inputs_v1 i ON (i.actor_account_id,i.command_id)=(h.last_actor_account_id,h.last_command_id)
   WHERE h.group_id=selected_group AND h.group_incarnation=incarnation AND h.head_revision<=v_navigation_head_revision ORDER BY h.head_revision
  LOOP
   IF item.operation IS NULL OR item.input_bytes IS NULL THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   candidate:=jsonb_build_object('head',to_jsonb(item)-ARRAY['operation','input_bytes'],
    'reason',CASE WHEN item.operation=6 THEN public.native_group_process_decode_v1(item.input_bytes)->>'reason' END);
   additional:=octet_length(convert_to(candidate::text,'UTF8'))+CASE WHEN first_history THEN 0 ELSE 2 END;
   IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   budget:=budget-additional;
   history_value:=history_value||jsonb_build_array(candidate); first_history:=false;
  END LOOP;
  -- Full owner closure work starts only after the complete history fits.
  PERFORM public.native_group_process_assert_current_v1(selected_group,incarnation);
  group_value:=jsonb_set(group_value,'{history}',history_value);
  groups_value:=groups_value||jsonb_build_array(group_value); first_group:=false;
  candidates:=candidates||jsonb_build_array(jsonb_build_object('group_id',selected_group,'group_incarnation',incarnation));
 END LOOP;
 captured:=jsonb_set(captured,'{groups}',groups_value);
 snapshot_bytes:=convert_to(captured::text,'UTF8');
 IF octet_length(snapshot_bytes) NOT BETWEEN 1 AND 1048576 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 -- Recheck the actual row shapes too: adding a column cannot silently broaden
 -- this ephemeral internal projection. The same frozen parser governs both.
 document:=captured;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   v_navigation_head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>v_navigation_head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    v_navigation_head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF v_navigation_head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL AND (after_groups IS DISTINCT FROM current_groups OR snapshot_bytes IS DISTINCT FROM p_original) THEN
  RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
   'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
 END IF;
 RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
  'session_id',p_family,'status','MATCH','candidates',candidates,'snapshot',snapshot_bytes);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;
$native_group_process_navigation_source$;
 END IF;
 SET CONSTRAINTS ALL IMMEDIATE;
 SELECT classified.state,classified.variant INTO observed_phase,variant_name FROM (
-- Generated read-only complete Group serving custody.
-- Absent namespaces allow historical verification, never arbitrary installation.
WITH full83 AS (
-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates'),
 ('deployment_operator_receipts'),
 ('deployment_operator_head'),
 ('audit_events'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('cedar_policy_catalog_entries'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_property_refs'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('organizations'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users'),
 ('org_units'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_results_v1'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_versions_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE (n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
 OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN ('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))
 OR (n.nspname,p.proname) IN (VALUES ('public','identity_native_group_process_execute_v1'),('public','identity_native_group_process_incarnation_selector_v1'),('public','identity_native_group_process_material_v1'),('public','identity_native_group_process_navigation_candidates_v1'),('public','identity_native_group_process_prepare_v1'),('public','native_group_process_accept_snapshot_v1'),('public','native_group_process_account_material_v1'),('public','native_group_process_action_roster_v1'),('public','native_group_process_assert_current_v1'),('public','native_group_process_assert_input_v1'),('public','native_group_process_assert_result_v1'),('public','native_group_process_audit_guard_v1'),('public','native_group_process_audit_truncate_guard_v1'),('public','native_group_process_classify_v1'),('public','native_group_process_command_guard_v1'),('public','native_group_process_complete_snapshot_v1'),('public','native_group_process_current_context_v1'),('public','native_group_process_decode_v1'),('public','native_group_process_deferred_guard_v1'),('public','native_group_process_effect_guard_v1'),('public','native_group_process_group_guard_v1'),('public','native_group_process_head_bytes_v1'),('public','native_group_process_head_ref_v1'),('public','native_group_process_head_statement_v1'),('public','native_group_process_i64_v1'),('public','native_group_process_immutable_statement_v1'),('public','native_group_process_input_guard_v1'),('public','native_group_process_micros_v1'),('public','native_group_process_original_material_v1'),('public','native_group_process_participant_guard_v1'),('public','native_group_process_policy_head_bytes_v1'),('public','native_group_process_policy_ref_v1'),('public','native_group_process_registered_actions_v1'),('public','native_group_process_registration_bytes_v1'),('public','native_group_process_result_bytes_v2'),('public','native_group_process_source_v1'),('public','native_group_process_text_v1'),('public','native_group_process_topology_fence_v1'),('public','native_group_process_uuid_v1'),('public','native_group_process_validate_frame_v1'),('public','native_group_process_version_bytes_v1'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_session_shared_material_v1(uuid,uuid)'),
   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),
   ('public.account_company_setup_eligibility_v1(uuid)'),
   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=18 AND count(oid)=18 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=27 AND count(oid)=27 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=216 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=27 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), company_startup_rights AS (
 SELECT
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=83 AND count(oid)=83 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=92 AND count(oid)=92 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=736 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=83 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS (
 SELECT jsonb_build_object(
  'native_group_process_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE starts_with(c.relname,'native_group_process') OR starts_with(c.relname,'native_group_identity_policy') OR starts_with(c.relname,'identity_native_group_process')),
  'native_group_process_schema_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname) FROM pg_namespace n WHERE starts_with(n.nspname,'native_group_process') OR starts_with(n.nspname,'native_group_identity_policy') OR starts_with(n.nspname,'identity_native_group_process')),
  'native_group_process_type_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,t.typname,t.typtype,pg_get_userbyid(t.typowner)) ORDER BY n.nspname,t.typname) FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy') OR starts_with(t.typname,'identity_native_group_process')),
  'native_group_process_routine_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),p.prokind,pg_get_userbyid(p.proowner)) ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy') OR starts_with(p.proname,'identity_native_group_process')),
  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),
  'required_schemas',(SELECT jsonb_agg(jsonb_build_object(
    'name',required.name,'present',n.oid IS NOT NULL,
    'owner',CASE WHEN required.name='pg_catalog' AND n.nspowner=(SELECT proowner FROM deployment_builtin)
       AND owner.rolsuper THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',owner.rolname) END,
    'owner_superuser',owner.rolsuper,'acl_is_null',n.nspacl IS NULL,
    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
       CASE WHEN required.name='pg_catalog' AND a.grantor=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
       CASE WHEN a.grantee=0 THEN jsonb_build_array('public')
         WHEN required.name='pg_catalog' AND a.grantee=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
       a.privilege_type,a.is_grantable) ORDER BY
       CASE WHEN a.grantor=n.nspowner THEN '' ELSE pg_get_userbyid(a.grantor) END,
       CASE WHEN a.grantee=n.nspowner THEN '' WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable)
      FROM aclexplode(COALESCE(n.nspacl,acldefault('n',n.nspowner))) a),'[]'::jsonb),
    'effective_rights',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
       has_schema_privilege(r.oid,n.oid,'USAGE'),has_schema_privilege(r.oid,n.oid,'CREATE'),
       has_schema_privilege(r.oid,n.oid,'USAGE WITH GRANT OPTION'),
       has_schema_privilege(r.oid,n.oid,'CREATE WITH GRANT OPTION')) ORDER BY r.rolname)
      FROM (SELECT oid,rolname FROM protected_roles UNION SELECT oid,rolname FROM deployment_observer_active) r)
    ) ORDER BY required.name)
    FROM (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')) required(name)
    LEFT JOIN pg_namespace n ON n.nspname=required.name LEFT JOIN pg_roles owner ON owner.oid=n.nspowner),
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_group_process_startup_rights_valid FROM snapshots
), phase_pairs(variant,predecessor83,installed83) AS (
VALUES
 ('plain','ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862','3f5972d2e5c1d7277e71b4f716b79a405d152ca5bbab25b614dbc1fc67c5fd7f'),
 ('observer','cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6','304e176d646edf62e8767a9f986879abc74753a3b2a8812c1a8cec8447a20b78')
), matching_phase(variant,phase) AS (
 SELECT p.variant,'installed'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.installed83
  AND f.native_group_process_startup_rights_valid IS TRUE
 UNION ALL
 SELECT p.variant,'predecessor'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.predecessor83
  AND f.native_group_process_startup_rights_valid IS TRUE
), namespace_absence AS (
 SELECT snapshot->>'native_group_process_relation_namespace' IS NULL
  AND snapshot->>'native_group_process_schema_namespace' IS NULL
  AND snapshot->>'native_group_process_type_namespace' IS NULL
  AND snapshot->>'native_group_process_routine_namespace' IS NULL AS valid FROM full83
)
SELECT CASE
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='installed'
 THEN 'native_group_process_navigation.finalized'
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='predecessor'
 THEN 'native_group_process_navigation.head_revision_required'
 WHEN (SELECT count(*) FROM matching_phase)=0
  AND (SELECT valid FROM namespace_absence) IS TRUE
 THEN 'native_group_process_navigation.absent'
 ELSE 'native_group_process_navigation.profile_mismatch' END AS state,(SELECT variant FROM matching_phase) AS variant
) classified;
 IF observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized'
  OR variant_name IS DISTINCT FROM expected_variant THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 IF (WITH expected_migrations(version,checksum) AS (
VALUES
 (1,'3a0113a4ccfa33c60918f873847653d4a23ca6a8e526e2b7389e7138f7041823901b9fbefaffaae3a056da8878791ed8'),
 (2,'4e7a83b1e764bb4ed9a1a18e8cc758f8b3343eacc12db6430a1aca17828ad3ed940f0d7d8b983a01eb9843be363ec32d'),
 (3,'43d4988151f9bd76bf9cd5f4306d25e41450efab4d8c73738839fd3c530dd91f547608b3327fd962640460595523e0cb'),
 (4,'b0677f4d9f7c52dd41b26b87e0ea8cdaa37ea128209e37649f15607034c5ddfe4c7fa4b60a52f5aedf7a908c56de3854'),
 (5,'4992bd5ed0da6dad0078d5b828027ae7a793c7b46add0a339ee5032a50ee1c618963348380085c20faaad592cc275bf0'),
 (6,'1d7327a051b404ae0578a79563a5d6a10590031e1d41d228cf7d64074192a40a6f0dbfd20b300e16b9cd1f0f87944115'),
 (7,'a1da5931f3b981ffe938dd12ea15f1c2a5a00a1122d3e767619752faea2d5e5e4befab42ce7c5ede4dc783754370d10a'),
 (8,'70e807747577343acd0ef1cd4efa36f279103442ad3a0a102eeed007da8d0c78fa414cc68623eda8a50e58806820737a'),
 (9,'289abe2c3fec02a0173020595f01301629c988272599e40ec0edaf4fc974189cdb267721fa01e27cbeeda38a4133328b'),
 (10,'145a8b5ee1ddda01ffbbdf7feb43a5b9b9a4a2149439384c0a6e049783fec6d9b25df84d517a2b0b21749ba29c42e077'),
 (11,'5e06932f48473ca6a6d9c02eead6ffcb78225678339edfa3c33ca3dc0d3a054d681dd92189e249389c8fb8c60b96d905'),
 (12,'2e7c57244bf658c89fc0b5be1e3ef423ece04afa7f4525040680e1ab0adfd4fb83c546031919feebf4f862df979ab505'),
 (13,'60d70cfa40d065e2369883ae2e19af771e69b491ad4f60432b1ed24f295bd6edda1323b333367bb0575b0fd3aa215b6d'),
 (14,'86ca8bc4ac10f840f1ebf27ff6f6e3a46682907f109e08a328fda2b84358bf8d9e6c307af83252aa4acf9353f3ea7852'),
 (15,'84523959c833801d4b094291b027c3f00d34f275a70bdb78467746d58f9515b84b3549e8df6bbe8f9f51a60216895b2b'),
 (16,'6fe1588630f882a6a9b0d834e9c1d7c781017b0970d78c0f00f239692cec4c20d1b5498f6d8ad0ca975f9435c828b11c'),
 (17,'efb61edac50e7ecb09814387cb141f0a4216e0bc606a17ee72ec8040990db06eef81100e1aaa87d4a999bbd8bd17f68b'),
 (18,'397304673b58370838ab79d3b6e9ab34e5caca708b6eff4c74defbbb3d9745fc61cfd5737d3f8d2345ba3fb08663f214'),
 (19,'6ba9d0fbd0f6fdbe57b956bf2536082e6f29cd9a7fc0c2d7e2988c9b0ad2226be4203fb944723ae48b077b4843b840e1'),
 (20,'b972a6fea540884bf715346245c7176af71ab3e7937c110a41d41c0eef7e83893b89fcc6e3311dd93271cd9c7d9f66b6'),
 (21,'de6c041eeb6885ebd5197c8bcda42746741b48dd2e288efd77086c516d7689808cd5ccecfa9d06959dd47721bbca7be0'),
 (22,'4d6d9a4fdaf749c6c963189184ac6898e21aded39fc8a1110fcf8a444bba062344d67210fe4285e4bf833541659234c2'),
 (23,'088767a033071d4beddf08ec887d08b99aee8f89aeae7719873af22c1d5238d42efd8737a60ceeb420962545583e78af'),
 (24,'b0dff6fca947bef8c1522f3e2eaf0a2d08bf9615639f5d20ea66ce1fd4feedb14cc56ac2c40a6a3dd1350b287b3ee241'),
 (25,'f5e6d37e2f1fd3e860a49efc1781712aa9a8033ce1f0373aa5825d77d67739eece75fef9b15e2ee0c302c7f247c74a66'),
 (26,'8fe15c2aad9d9758f63d71bfe89d292e1d37992e7a11e4f2203a8acb79004bb45a25e8d425c542e2df17433b12c227b4'),
 (27,'cb7514a726fac887ff29aaebfb109d6936f96b0f36afe64d4f16fb37881c307265434ff7bd04ca6c303e695738974b1f'),
 (28,'53ff45d0502e254e32bbe254390746cdb42a0cfa412a6769db58f20c2de6075bf08a328c10051f1b62fb92618e097c15'),
 (29,'f286d89243c75ecbbf67daf02e5314e4eeb12320800d80ad7638c91d6f1a65c76d734e6dae88fe3b28e95a32355996d3'),
 (30,'2d1ffb98dfb9d486747f160a2e73578a627c53f56cc73ca88353bac326ea6b6716e1e80e8a54980269727359f164315d'),
 (31,'c2c28a143b49f066b49a6195e6d1743868440494da06ccf59a9acb75fee8a6ec61b0256023e454854ecc249244cd8eae'),
 (32,'3d60ed94b6202318e01d03b05fda4da3a165f8ac5951034f609af6b40add04973a1fed178aa95b6afd48de261244af5a'),
 (33,'af0e888da79d2f59b6092990ef68bad5e2f561e6ebb7fceaec25be1145620be17cd0f45683d7972f3c56c3c52d08efbd'),
 (34,'922b56030d6efaf35d6a045e3420c9dde8b30185f604c1c9abd93819486ccf909d082d773d9ca7611b7e3d4a5197e3af'),
 (35,'c23ab31521af0493b15f3d5a8d7788d294e8bd8523da860c78f829ad5f94382bc27150d3e79a02a1ee797b9d6559ac4c'),
 (36,'130788c966d5f4dc844300ff4e7fcde9abc066337315ab4934d5323ac3fa3e62a8db823b422c7ac82e6291bac4b4506e'),
 (37,'d2c69725c11f50fcb7e9cf48331b00a2d5ef85e6e5828f381d004275bce286eec40142bd25874f9a18891ff7f9168ba6'),
 (38,'c1d4dc26af31ea053f04694c8d90565a7265f5185d100807be935644f7e39bfdcfb73564e10cb784d0fa8f3a18568dba'),
 (39,'6f964b1a786bf483fdf3d278304c8162579f66390129a77b1e91a0f9ed014384e16c2a353f6eab2a75704731e5325949'),
 (40,'5c48ed8e6296349d7a818045342b47001ced8d5780be040dbe08e5d98a359c98ab6e214520a841ad247737f36e4b613c'),
 (41,'a0d8622ea4889b69238b89501a14f2f3dad3fb8c6a1f7bb157fd554a3003aea541cea67e573875a94552eba7ace77b5c'),
 (42,'7ebdbefbb132716ad6918f45b59379416134071af1613b230b3b27f50625730e52593f80539a6f3528601dcecc855d3d'),
 (43,'8309eca6bb6f85fb56c50b11bcabfbec53d1c301e2037f22e7806534b947758278545edbe03cb91b754064e066e59864'),
 (44,'128e14ff338d461b9c460d69bf42d9dd74832bb66e45ac93a093cff47fc47761a144e3bedc790b9916f996b11a470ac8'),
 (45,'aa5dccf7098f2191e241dbf58855516350e794d410aca7d3c5894e6e523c346afbb487f7ba55074160948dc87388227b'),
 (46,'cc63f87020e2fa7d897bd6d4ebeddb9c29301d3a8785608aa84f57ba658636b9950a2a7570dbd7972a81cdfbfbe76d65'),
 (47,'c203f0dd5d26d61e3cadac84a85839a0a5705a68f121a4ab699f890949eac4c45a3c9adccb7149319332438966b170ad'),
 (48,'ed500a4b42b2ece253133fb969c1c8cadadbe4a12d33b3b11919acadfa640b43a1ee78342bc4a3f7c2203baf1a9dcf16'),
 (49,'ce431e273e6b0ddcc29fb8ce85e5faec8f1788462c507da2671e165aca8d169e23fcf3c90228dcf2b02888de259984a7'),
 (50,'3ac1d0105e675fc9818a2aa62819e48f6dda1f60d28aa1e366e95c43a9d80bce9218bb20bcb964a2c89eb291838c5cde'),
 (51,'da2b69cf3bb9ad3cde6abd210ee4fb38ad20a07d367bf30f4090d2843e21c5661f6d8b0a29da5aefa5ef61b329b9ebae'),
 (52,'dd3a7e28493e590a7f7dfeb8e5f2ad2c055914752678dc9d0403332ec87ec2e04147c1cf41cc752266ca20b7409ba5b1'),
 (53,'95416edc9552f7a2a75b82df8cf56209d14cc99e65b44b087f9c1fa7b0bd08be7741084224d696dfa7a562e12fb0e8ab'),
 (54,'6cc66d2baf1abc03e0de50b942df746fc67cf93bba3e1cdecb334843b253b5823a514459fa7e50aebf59c21813d742b4'),
 (55,'f729720690f3f1fcd6c74ccb3d47577ae464c2f8547a24e3f893cf0ef4a517849adf42631092fe1ce5818b9c7ec10a3a'),
 (56,'bff90795e98202b2a85b15bad55f29f321123f1819ee9ab76b927ed215257bf0c3bb338cbb9c234eaa3c697edee69ac8'),
 (57,'7fa52e9563ebde699a1111801db42c2dd15b10b7002b9ec6d18f3dd9adace5657db4610683c6253c62824ef4a55bf173'),
 (58,'fc2cba6a8225e9f20d7544378d53467670c339858198d8021f9cfc4d51ca0074c8e610894854b48076577583ba840097'),
 (59,'41417298452d185d2c89fa1f693c3f35fbc77295475a2e29389b352eff31996de27382a8a3a23530056dd1c5a6306928'),
 (60,'11fbf43e150c5ae0bc3f91c10d1ea2c3d110d586ad956f32a77a23f9c10293066efb125988e0f76d0bb09247b11a632c'),
 (61,'f12dfa736bbdb67b5b98f394109cc02f41a4d006ed1852e36e079feca60e13538d2f6aecb4e9f25984011a6e35da6cea'),
 (62,'cc76fa13769e8d10fd504b28bd7abf9fbbecf4c7d129c5cd165c837cea3b35735c105954ed3809e790beea5ea35593d9'),
 (63,'34b38ebc31460874b3342b9f9ffd0fdeb46813075b547d5d6e9dd9665402ebf691e412639a67a4f86c8fa308daa5b931'),
 (64,'e7103b2db48a971f7b8fcf49b5f6676ce6a2cbbd1e70e7e0c1e06324d2e6cfdc22c400be283e2d737a4515ac5b1b3903'),
 (65,'26687e7e3f9cc7e1079033470e48a1f6136665724b9e81c2facf01e968e0e62e5a41bfb687ebb3736325d3fb309a8ef8'),
 (66,'f72e67e6bd8bd9e41e26aa1b94a97e8d41099c9d66ecfc8f888d5180596aa808ef608c29ccf972755cea551f196ba187'),
 (67,'a049002589b2df5c3aff978c1c2d0b16c791193b512a0cb8ca4aa2eb50b82b65ce6b8daad6db4e4c6ec6ef48113ffa61'),
 (68,'6a5d5e141b07519cbae4eeac15479a1d657bb70b6eabdc886cc29b1b9fca536176eb1b232bfb438daa289939f8d8d2f0'),
 (69,'9984b8f83f6e012fcba1eb9227c73c428023ceca1c9bbe76700893b63bf9e438f277935b8b8eb9929cbfa4cffd0766b7'),
 (70,'72f25fe770b8fb3bf99cf600f8775365cd2b2ff5a57ecc57545837b05f5465a5fe6318dc0d37eb595990bbff1131bdbb'),
 (71,'ed6906ccbb2f177b7c6ee102e5b9ad896c952f635133f69b3c6e8484381d7d06710197012dc9ca646f62ac656be69313'),
 (72,'61d85a437721934ae16988ff103c696b4ea144e9415735139a7732ae005bf94f3bf2735139df6d3c2492f6eaea02c800'),
 (73,'514f3d73f604bb9bc6cc8961f9384b0ebb2eac51822a59897062b3acdda15011abd003585d8368865d64849a9a251c1a'),
 (74,'eaa12d5a5692fad09614b744cfbe031894d0921a68fb0aa89b5645948ac430767fdb3147bcbf8138400c2a9f7e338782'),
 (75,'9ec0263ad251db8d8f7819bfb028e272dfc7f7975d7b66d247ae3285325f6ae790418933267169dbce625c55d7d10d23'),
 (76,'84c9a1e613e5fd3a1a5ea613e997aeb2bc7771ff14efa63dbf238a8bfa0ab52127f73b1834fc49e95b1772dd968f2547'),
 (77,'d9e34622276b1e33fab09a2b0e2984585470b5a85ee5628fa2eca99d783d119cdee53bebd8ef2a565236a787c69065d3'),
 (78,'3c4109f418a7673e36f37935322430361ad21b72d5c85dfd2cbfa6b1d8a131ab699eb83bdd957973a3d30b3c3c277942'),
 (79,'ca6dd7dbaf31fb4e4f547660cf97ff08dac527907300f6d95c5d0207f0c03c761f9faa236f18b0f38f7300d3efd14c01'),
 (80,'28f736f00774fbe801dcba751164bac883d565e61040a55c0da3fc0ba05c382f189a1c3d20ceda41a6b17cc9f0207004'),
 (81,'687409fdb5576a6cbad763d841e66390fa4db82e28a756e54333abc3c90fac496a1df75f06281a66674937f034a69afa'),
 (82,'dcd1ea86570b5ef0bde9a705dae04962be8546837990df5aeb9eb81aa8f8810064f97bc0218474f7edd52e9ed7aa0036'),
 (83,'07d03dfdb5937cc6832f77e6b08b2083ce7e78cb278f48f8f764258fbd67b9d25dfae8131c5a88e55926a69b8341a45b'),
 (84,'df619f2e351ecb478079a8bf258f8a9a9f8695f13c63a3c8b06c7f24c280be2eba338113fd4988c33f864300cc224415'),
 (85,'d44c2fdc738c1b9165f603a1946ea5248bff1087625798697f5ab769f8933efe9291230ce553e9baead903351aba0662'),
 (86,'50fd4717398ed96ce6f942195240dd3d570167910c46a606d6284ded85740a84fc688e84c8caedca839ceb0ccd344820'),
 (87,'b0b787ca697d56080790f6dc79888dd600c5460d813dedac9c0accd480c348c4351ddcd5e3a0d7056613d7867392ab45'),
 (88,'6d4e843a4175afff9b0ddc141c10da20dc84051ecc81007e977d17e5b34fba54607a5450628ad98ab3531c0b41ac8392'),
 (89,'bb5cdfdcd60d2fbf3419b40bbc5faf2941044bb77b6d9ac4f0429a0ec86a047acde893d7a67a089a1807fd6675d3f8cc'),
 (90,'2358c18738aba8535b35f36b66c625b67ee581b682fbe09c83e0f5d41ffb540d7fe4cdbec3271d5b5b75428145402d7c'),
 (91,'e353a91a58d7798a9fd0b1f9c5d78f8392290c23c6702699b469c0ce6882ff1dab502fbcca37dfcc4b7af454e2f78ede'),
 (92,'8c72f7acdc95535ee14d8d20031c5dda5fb9e8b10ed6a849d632e43c88b9119320568c769f295b49f6c2d42e0a41a783'),
 (93,'25a6e38fa174f8afa91da34c8aa2799bb98e028b504f1d67b74fdd12814b2046fa9f0f1b2f51238dffda8e11ca2e9b18'),
 (94,'c1f23732db4d7287cd477d95fc5d6a39ea5dac2339063f3d468df0f44e3aba8753013f2b5663156e475820d47eae5d21'),
 (95,'0a0aee53c52f05699d5a66f9e83be3ad087067379d1ce64d157ae6cd28bd4ff9158c73caec68e8b30e5ca978a393e9e3'),
 (96,'1691062d3f7045be80b93dfadce4b82a40f2f7afbc926d24b22f6c5ac4cfbe566b4265cc5835a0c5191a30dcb7e674f1'),
 (97,'ba786150ff5f93fe98a88fc33d355598cb55cdb3814958cf0075a558a94f0c37bc9c75332b4b991ac09aa8bb8bb0eab5'),
 (98,'63aa28abd0fa597f7d3947384071edb2619d5c7b24f3796ce066042f91c06f88ebf361cdfbca7ff6ad08e7151e94b31d'),
 (99,'c1eaa590646b6ddccc38ab93b6cb643cb854bf29e8adc7435c8358a1da642d0190a741a67fac0d330a9404672a3299c1'),
 (100,'0edb07ded15bdd526df21be47666a1f4ad49bf96b9b250164558e830447aa4a3375290cf1d8ba924201e51cb984c5fc6'),
 (101,'060c821dbd72fef7eec2205f0969b8b83279a9de7b9b1e0fdb80ad49b1f1aa8566fab36470c8afc49095a1cdacadd2c1'),
 (102,'11504766954246ab46d0d70d68d17aa05714d5fa8a8d605ab8b022bd352b87ecd4c6daf668acb63f1defa09090e98c43'),
 (103,'d8db143aaa927e9e587244dfc480fe9e7956a7e93364884728dffb9abb7c15fd27fae78f01de8d32af0f49eaf1a6a3c6'),
 (104,'45c18f72ff37f26f465ec172d2ec3f060f9610e4a44eec85e27febf66e6a5d95201318cf58dfff0584883d12107b592f'),
 (105,'86fe155ba98451f2da9b39ded53dac1c4e74e290f4332f8ae01c69d40ff06278d7f3adbbedd3d9ea3cb1ea5314ef6c5a'),
 (106,'9046662be807be32ba9dc9a85ac51983fd44da659b3a80cc2d30a722ca00b5f2b726fa1686fcdca8b52bb80af841746c'),
 (107,'5e953318b3f6927e8164ba021addfd29e3f3eeea15ba984e582a90f4f3618362e4b94865c019d7d833737b574f472cf2'),
 (108,'8db7c0561a22d5319382b9debc193b57601f51b3567025c652d18ca40f1c2d8a5aad2ce7c55f15e8058a5b52a10db014'),
 (109,'160b1db4d1448c35cef46f0d7baceda735f021e61a4dbc575970712fefa19e31656d9a8eb7f4bc4e8af72480aeb89222'),
 (110,'8afeae0471167a5e50d6e165eda503423eb3ee8bd7bded4f4fdd3c79522f3e97024402f3f1d76f6dd0389124414de37c'),
 (111,'4cfdd074aa7db9e1635c039cb3f18a408ab86b9c512f8adc1954ef5ee7dc41faf3e3970f5a5eb9d60b086c23b9eebd6f'),
 (112,'e3711ebb243c5fb482cbeffcf4bb940f0d79c640d349147de34592930b6a183d3584ec4331da1264ac6c68eb5bf89bd4'),
 (113,'680c05b91798654ad570b650586e7c8eee41d7e8c8d02e3060e633a93ff651baf08030f224e35251d297a454f030aa6d'),
 (114,'cf245d1ba5ca7ab50a4d2d66ff61a04491fae691e0e347f3b73f59cefcfeda4373463581ec2602ad7eb09d4821cce0f7'),
 (115,'acdeb9bd8a7e51749c7bc7f025cfa484f6435127b62ae58c7d84a31122e3f94e6368d2a809ab25acd2e2874cd0d9577d'),
 (116,'74650d0f824239a1df65ca6ea36bac83f326695683fa3f764f5613d6f63e4f565726fe3657bdf3c367534885c7fe8768'),
 (117,'95da2eb1a58040f1691cc4c332b7852140dc7953d59aff10ce3a91886009b8eeb16573a7e79ec660c73755005be02af8'),
 (118,'4a32217cbb2fa56ce7e9d630ef2f3c9981fee5c2a2c7ba7f5ca1a6562cd7f8a45f10d8e8823f2dc325d95db28430c7c6'),
 (119,'0c9a590b1c2cfc3e72285a33c885c44a53ad8e6c36042b1198f194377227272b5d4faf7ebfa7fa71865bc8fc1babea5e'),
 (120,'16eff9604df617accb4357d25d235f4129a67c0a35bcb906d975ca51559ca273c54cfe61dfd6007e1b95c9e973f295c6'),
 (121,'19c3078e6a2bb1dfd1851d6309fdc4039919652e5c47757905c586a712a1753a4134e23229b7f6aa7848ad5c8320f511'),
 (122,'bf9f7b0fad7d7992c227d25b4f74457b58cbd546fed3a9d9e28786c19f381d1e8d194f688f978d7717f1e918915c84b5'),
 (123,'518f7122edbdfbaf717a070805f4581e1dbba85ef440de8ab34d537893d77de9331ca6816bbabf1ea67a4c4f4c0cc4e0'),
 (124,'6ed48f666b2ecaa7464caac3e13256448081f3e0b0176b101fec1a938e4f5708bd1936484809a42821d92dc455e47813'),
 (125,'8aa0b5f9a3464da1f7a0b58b80f9961724c68e96421ad2d896da940adf2cf2d95053da0ed42d78dd7b99e9a86b734196'),
 (126,'8709abec78817f8923850f3ef9c10f5248ed288227292a727d288190e110fce6a053b50c44e3ab67960cc171fba155c6'),
 (127,'a32d11a9538d9c7d5c4ae977b7199debfb0ed891081b9e91b34e6f8b86657f2c17c32da7f19d8ee5db649fb4cd866888'),
 (128,'53530028dffb8ef5f54df23db35f338f806ec7969f3222f9d403e540874c6b2f5e8b331d3ffff240ef94eb3f0d99e00d'),
 (129,'6e8650760be23bd5a604aed1bfee4e05b4f9e6e1fff076ec6c4bca1890c227129139b0848459c3f76c45c26af1cfae79'),
 (130,'e3b92a4197dcabb7196c8a2da673023c45552929f26ca7b8a97f6e32546b230f59fae9ea8cafdd9fe0b61094674bb0f7'),
 (131,'505378bbfbe858d216adea7680d3fd719e4e038a6c63f92217dc65aa4c3b83a07eaf56a71b4a6cc8a01889e26b1edc33'),
 (132,'0baf95c1028c5bc3c9d65093b3b1c44c37878004e500b626d2520be344093ad918f929b991c9371de3be55dd6dc27cdd'),
 (133,'59a616a449270e4ba589fb082e9b24887a194fbf2afcdcdb398086552bc24db8908dc3d603468f5b36ce43647f20ff33'),
 (134,'0c9112c2e127e0941f9cb1879d28affb753fa174ddd31489aa350975ef4a038247de75fb747a30537e8d5a12edc7d752'),
 (135,'bf66839b1cd85bda3903f5f157ea1485ed96035f86b347381ba75b91dcab2f181443d903793f620648764ff47f09adbd'),
 (136,'f31df5525c33e814d182d07233fcb04e712e508bca779243e3fd96fd878105f86cc56b563062a28e3fd4ae36c8c84022'),
 (137,'286cd33b30945dbf918750fa070306e2c59b0726e4d34652857c84731bf77b4f3d95aa64fecdf6155286c154a5975b63'),
 (138,'4bb209f6d2122075778e55a3a4821756e047783e1505f96463764775e6a41651a6ca70d46e6e5ce3e4a15fb13ac4981d'),
 (139,'1d65325c1e3c952635858624a0e4c25af8b242a140ebd0094bf0a837d2ef99676ea45bf098ca67ff65ed6a0c8e4477d7'),
 (140,'d68107b087ff37c1c0ca82fff7470c2078f0aaba5df4b3e5b0c74a884a0a2cb084d8e17aba01eebb0506269084f0cc08'),
 (141,'d54ab02ac2b87b957b7b950d9392206df319ab45f1a996a3ba30ac6497f0cc0945e56d6fc51e4e9bf39bc1ed64863158'),
 (142,'a08d851fa096616987e2d8c16f2fd8f8e30eaf63777750739069395f54a43c860c77346bbfd1965652149fde0d898e5c'),
 (143,'b4f6cb9064edcf28b34b11f0a52f87359ba9bc1e55b6d114d5172dea4851692f8bb7f5f8e46c3e68b7ac78afd25eb0e0'),
 (144,'b3b0e4a241d8bbe72baf914c25648f4286aa3e4d6e93b28d49cdd705d16492a73ac3550511ad5a1cca26b828bad9119e'),
 (145,'bbe9612542b3af873ad0be48585f6d8ee721e442c91fc755a62215d4f9ae94ef9742d45a0b8709a4b66c38e64222ddda'),
 (146,'e9bb5a02368a1a9471b2309e0bd0fb41b2193a7f53c51c941e71bc51dfa0fd0331d5ba820924548bdfb207143bd91686'),
 (147,'1430b4e855fa35898abed6461637bfea3cdadc3e1f9eac160d28dda9e82cdb327927e6887566da430c5ea55e6acf61f4'),
 (148,'7cd80443e400b82b5196aa6779976bdf7271ea55c651086d8592f4b5fc47e21e744a3efa3f258585d11cd08f524490bc'),
 (149,'72b9f1d0ae6bc5177c2421fd2aa3317d8895f8a667475d373fa0c4baae09525379376d4d7acd615bd29206b4c3fc17ff'),
 (150,'3dc30c7a860a66f789fa046ccc5d4ed7120cdd6312368cc785dad8ab10d94e7bf7f2ff7e4020a8ca5ab70aa946fdae22'),
 (151,'b852bacf19c3efcf707430614a29dc5f5140de84d8009db4305c1e1373ad8b52dc4e5a43071fddb840b2ffd94420f497'),
 (152,'e908aa1fac7e254aeea8024046c180408333643ba62931ddb25428f46abd33d8937d7f2176115558a3023b41b7466c9e'),
 (153,'cf94ab91fe29ea304cc994b2a5a176a62b65bc61d2aa49d0213163d9519bd19c4d6806f236353568f0976ca541b0e722'),
 (154,'d0c2cac03824de68b11290d8bf0425740e11aa17541156de4098f0d1ec747bfd146e09d3e312fffd12d6281776b2c68e'),
 (155,'a7864f1f67f35028f069cf3316580a29000e1383081c6d01895dca15df085dcb2bcb115afff5ed4b9162a4ccbcc6eba2'),
 (156,'fe8f2f890b0ad5a5a8d51deb62136b06d41d97575cbd88e8b615f5121222631930c0392258f73735ac2c55bbfd4b6838'),
 (157,'90281ee5098c997a38c26b14b31883e442dcfe5a8328b86f698d886ae8a401c40635430eef53a6d5000f95f6925d447b'),
 (158,'a6ff65b3c5caa1e8918bd8cf85fab744bfe57d99c3e08d652f8044b36200476c83f86561adfe6d69b49d68a46cef974b'),
 (159,'3abf97cf7ac868893382ed2f7d4b2610dae486409b3394dc4212cc30bc7971b16dea8f77f113e04b57b660c3d67fa400'),
 (160,'1c2478d1f50f4d69a148bb8bcde17a137e652784dfc5376ec76d88bcd8a8dfaae7af4424e2e1c89e7b05e935bfb57fac'),
 (161,'e8135e74041a52b5fbeefc2fd05daa97b56ac4ead46a6b5414a167fd662a4372a5be6c2ce4f73f1330bd15f62c016903'),
 (162,'6194bc62cfa25da1dd5dbcde7f66935c915c5316410ebc44db03619d94e34d11b1680e5a40c1e4de20684a3dc42e302e'),
 (163,'1935b1c285de4272b9411bf573361fb69842353f633f4192fe6e50aa5ba54defafdfe3403e828a20a307b35684907e48'),
 (164,'e2eaf80d402d9830adae0daef03de59cf4d7af12952b447edeba8b9d4bb09c42c418869643cc0a453df4e2d8b94b6660'),
 (165,'87cc0c80998192f65c4fc93a1705f252fbe62126e32e314e9f49250a4e02853998bc113a7c2c8224ebcea828a0b1597a'),
 (166,'0fb85388a8449229eaf408af11d3f60b34686ec95f049afe210563d98ff6294801889560c3f3c896768ec9227e834242'),
 (167,'095525e24340d56e14b19fee28a7613da13db7439f409dd7e364fdb6d0b798e48982adff9e3bcdd392c861a2bf78033b'),
 (168,'975abd1ad8583d0c2a8ef4209bb0e0e41e84cb020e7b6688d17b187f02d20584052cf7e078e2048f86033c1ad06ea54b'),
 (169,'495bc0142d574687e5d030694d1ecde0d5c39234a32e4ccf631c1bda854ccf02cfae720054915587f44de5dcf719f372'),
 (170,'8f2a88f3a2b21e87a4411350fcd3cb3dfecc831a20777695c9b101657a8fe9f162220cf893dd80c181bd506bc7df8187'),
 (171,'2b6dfc4b21426169494fb47c707264921aa23e4665ca02d0b67ee23d2d21a032bdb973c97343478c45a5ae1210b09958'),
 (172,'e58b84f06ad6b70c33275b30bfd71d4901db50852f4e18200d233bfeb91ae71b450d46c3f98ed60f91cb72c54b90db98'),
 (173,'a34294e2fd9748be13ae38fbe3b6e654855047978434f7a96c8a6d0b6feddd6e68400bb342be1d6b605f894d34feba87'),
 (174,'a7cd3b6285c776aa80c2d2ca3a782d4756f4cfe2f510ca82456b458047366ce3682bd86f0dee2871fe60a298acafb8a3'),
 (175,'020dec31fd171507f6e0a1f4aa6c11f79c6b5c05b574147bd66a23b17bbc8646c44175bc19fd750825e7a9590c9dc857'),
 (176,'dfb9a692eb09529e9a46377e93db9334a2ca479e79ef5465ea027475f707824c5540fd8de3a12b6069ea7adb8f5a1c0d'),
 (177,'8c2c752d556be30c95d7814230b3228d3b06ab4602bec5e3ca5445dec9e14c336fc9d9c354e2bc20a98e9040d96262b2'),
 (178,'87e1bb9723711332561b39b465fd0a8ae63187a3c987a055ed9818648aca356d4e2ad78f38052f59810680b3d9f038f5'),
 (179,'0ca74170689590871528b513cb26cc23f004faf06011f26b4acff1c023610590ec8e260d6d792c3547ee17af63e7be47'),
 (180,'e8a92bd717910cf122c9afa8ffb2bfa0e4c447415ef9c9aff6421f1592e9ddaef85699bf8f4e8cf046e85e5423d07971'),
 (181,'05b58ce9a18af92baf88361d82ae3249f8cf7d2e6c445b28f475e81c793ab5d3e273eda853de4fed827f58185c2ebb58'),
 (182,'6b161e747cfa1ad202b0e1da6e543bc164fc2bec812ca15ffb8d65d10cf44913d8051df30f8aa980ca916aa1296956ee'),
 (183,'fa3626d35b48b995f3b1e90a8524d4f80cd23ba1810bbdbaec1da6b8818a6f4f436a9dfd155f06fd701ef2fef01b0f70'),
 (184,'740e24bb50ac61cbf0f81d6966885845c72920a67c93b5bf62103a7387adee0902d71e0c4cf3a545dabea4abb245123c'),
 (185,'eed0515b89a55f717ded8d298cedaa398e45758ae8b7f007aea3839754e4a87e581de7290b84881c0d9afce979a291da'),
 (186,'ac99f53e6506757002576eeb1d0ac20fa3495b25e57d582e8b24ba72b3feaabd66937d13e6baebbc82d501eed69fd913'),
 (187,'51b185bc4de388c11e449e9e22f6730e36255bac2c706c47804047ce27378acf979b5be711d6d35e38d8214e23d18d24'),
 (188,'fe0719dc44e3f88409e6f5a691978049c6f240d5980cd97e1df9a56cbdf321b57a2849d36b9cb5ce215839af91e08aa4'),
 (189,'2e97dbe2165a41a8be7fae6a1a143c9ec99e12b272ccc42262763d16accc278dd635b4bdfbf66f9c683f000a42ac3730'),
 (190,'a9b31e217401d66c035357a2ae7e06055ca63d3e7d1c3fccc406785c5850b59954b8da5c87b3ebba286ea56c2fe5379b'),
 (191,'821366ffbabb98340336552e9a1eae16331ebface44f2af274eab44e5c2dbed5449b5a7a2e7bac6138dc24f75043bf0a'),
 (192,'c144b5c1b705a13195ecdbaa7a043f6d033254f95b89284138be156cfd41c3568045220266d08188b2e2d55e568fba99'),
 (193,'c6442171e050ff3418553f70121941ae56a6042fc69bdee9cdfbeb150bfaa3087cc6d314f3c9d5fbb6c33cb9a84d811d'),
 (194,'6636777e8f5bf69a444287536b254f316fb000c9a7bd2075fea83b9cd1f3951360783748878903d3457c699d71057b61'),
 (195,'8080263264abd05de7f2fe9253a2df76efa47690e24676f28603462bfc436f25c446959ab2f85bdc946e7615bd576d61'),
 (196,'6dd0674ca2f11656b1a04d3f6bd47e52713fc03a1345a3b37ed530995c60cf8669de08f75ad6801069be25c39060d402'),
 (197,'3d01ba57771ebdbbf86e2c5febbdb22a9844b9c3dfb9bf45e0b8c4adbf04dad347ce37d1d668a045939999a277190b1f'),
 (198,'2d75d5004d48d5d759b1a2fb6a62824d490139fdd819f57697fc07616d7e8691ff63a1218f61f058bede0112b397aa0b'),
 (199,'5d71359453e86f74cf193b4a9e36002a1f168266aac99f3fce81472e1df394022d04250b20b54dcc90bdff20a3673ab3'),
 (200,'88926bdb190d3fe1203b0876539f0bbb095a0db6b7fd99fafcfff94c578383c90f80839ff8acc563f8abafe330ce5944'),
 (201,'3460f07bbe894727aca489a228e14f68a3808bce67488d511d462151d31c14e2199d19a62425a41059112b1f755797cd'),
 (202,'44188d68d661f6c54cbe879d7c92ad5daa5e4b5296ae8b603cdde7b6e21d7dd3bddadcb5d5fd4b176c6f73a10afff9e4'),
 (203,'c90fbfdca89246a5f4176ec80ca81eb81cdd4279c9a0e6f2c2a225e064b969620c8516803e3649e68615e27c98b63d7c'),
 (204,'373fbf4498192612acf80bcf5b297fc4554218c8e798c197c47740dcf0284b49b4a674685691c52f81a5689a0a4219f7'),
 (205,'c385adffd077ca58fbf7e594efcf29a556b9d06126b093178b738f67d759ec8f663802339d3c54078deec8aa091607ed'),
 (206,'9564e5fbec6ca945d9e588dd6c397a456ddfe0f5ea0d21040779d9b00a4edf4c1935ed7e371df33570f498f9302508e0'),
 (207,'f316667b1bc7d5ee271f325a7ca05dacc00c81c01026e0f1e50f805813e61ed76515746097e77ac45156907a503553e3'),
 (208,'67aea8e55a50b144bd212ec86fc395757edbd56a775f53bdd96da781616f6fe85c580dc4470d0965154d92e4912fa744'),
 (209,'2e3579ad7996b38b11dc67f628f5b2e28b4498d6f7d91c794a577f2992d1a106769f4b3e49b25ce0bc7e01d526816cd2'),
 (210,'437e213d08b274a6750c5c5c0a1c47db0915608ed1be57595850050128632c6a2922ea8bbf3c51f296271ad11521cb04'),
 (211,'2a4762c24a876378086ed791de328ba6be9d14b73c6da819f1988ce59a440f6a91f29f4dcc25e7ce8d1bf3e6096c562a'),
 (212,'e39a49097ef8eaf3cd299c90c968cf9373dec621cafc06fb5570fccea3db8af940d87e813169ca37b1af7bb0504b42e0'),
 (213,'96a735bedfcac3efb6543bc34a53f65b151f017507ef15fa4b8b52d82cd158c650cb50e1d3d0e0ecae444aaed564cccf'),
 (214,'465efb6550e8bd62083935ed5c75b39a140a1cfb151a4adf5fea3ac03eafc07eb238f137ac77c1ad8c4a8e91df56bdf5'),
 (215,'da27244d17d0e3d2c864148770e32f4e35fff815c67dbbf2ac18ef89d0f644c700b779677232b75149e7de261afc252e'),
 (216,'ed95fd6de7c6de897b8a6b9fb1c93aa320fc23d86184de5669142dda8a7ebe1e7ca4fcf1fb67ae7e79ff7b6e308ceb7f'),
 (217,'74120cc5319976843440b25208c448f9da28857cb309844efa6de29ffce734d6c8e762ab1538ea879554cd17d48436b3'),
 (218,'53c4dfeb76cade36b76bc36db6ae261d7148b886e484293f18737e8af803c359d5cfeffff50d68fa8909875aea8dfc2b'),
 (219,'75905740323952bd207fb0bb6822afbf365b3fc3370e877c5a305f87ec8ddc525bcf257c7318ad3b8bf2e5152d830233'),
 (220,'2ca34d16a157ec631d4ab599b77c27837a9241070ddbea3eac709ab45cf3ab298a99c84931924399d1cfa2fa070246ee'),
 (221,'2d9e65b180e8cee447684d132348f8a80193ec584687125b6e9a07de8cf0a5834a5fa67d302c9466651f30444a961852'),
 (222,'fd70d4ef27c1503829f61bf4d16e45a3f415ebcf561fabbced0135a5bf29868e3503dab652630a392d319192af13e891'),
 (223,'6a18b70032c039d930c6b051413efa247734d97b336443f99ba0ad354be431e1084122327ce055198296f2a4a5d56b6e'),
 (224,'c7993050fa899cb736ba8f179b9f2f0ba4dcbe8062401317fabd83232e507452d76990732046ffe8744acab035cfaf45'),
 (225,'5ac72758cfa1829a397d7e978e4bb0bdd666d0384fc5a9b10d38acef84cfa502b06d69fd5ebdba1cfb6f0216019f75fc'),
 (226,'983ef9cb798b0d15938080502a3aa09ef6f3461bb4ab686f02da484ce02e188d974efb4850578f73c9cbefe73fff9102'),
 (227,'af0cb575b06032a376082581c1b745ef11590629b8df761ad0eec4d949fcc854efaaf6724a990be8cb0b210ca50b0b37'),
 (228,'b02632c8d3a8cffb059055d54988ad68b19fb06045bbdf5b40650f8c12ab0434454d2b1d1488831a564072573ff0acf0'),
 (229,'08ece6d6edf558d06eff11297da18184e9a762ff4f5455fab135c84a1b34dbef449b6c957f3d7fce197af113f821a659'),
 (230,'69b9f0de4175868ce73f01ce52ac47ee5607ae6d88c91ab9bc519112ea1cb9682945c2cc0b87a91a78edb32a2b3fade1'),
 (231,'84ff6af6b0a607b596a7b9cfb126ed10c7ab744f6a3abbb005569a1fc182ec055afae72fb9c5b793bee376dec316fbe2')
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_mismatch'; END IF;
 IF locked_relations<>83 OR (WITH required_relations(name) AS (
VALUES
 ('account_context_candidates'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('accounts'),
 ('audit_events'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('cedar_policy_catalog_entries'),
 ('company_actors'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('deployment_operator_head'),
 ('deployment_operator_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employee_person_bindings'),
 ('employees'),
 ('employment_revisions'),
 ('employment_source_bindings'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('leave_balance_import_receipts'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('native_company_property_refs'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_results_v1'),
 ('native_group_process_versions_v1'),
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('ont_action_command_receipts'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('org_units'),
 ('organizations'),
 ('person_revisions'),
 ('persons'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
 ) SELECT count(*)=83 AND count(DISTINCT l.relation)=83
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.relation_locks_missing'; END IF;
END
$native_group_process_navigation_custody$;
