theory I_ModelConf
  imports Main "../ModelConf" I_FMT_MSA
begin


record ModelConf =
  modelconf_subj::SubjAttrConf
  modelconf_obj::ObjAttrConf

definition nomodelconf::ModelConf where
"nomodelconf\<equiv>\<lparr>modelconf_subj=nosubjattrconf,
              modelconf_obj=noobjattrconf\<rparr>"

definition model_conf::"SubjAttrConf\<Rightarrow>ObjAttrConf\<Rightarrow>ModelConf" where
"model_conf sconf oconf\<equiv>\<lparr>modelconf_subj=sconf,
                         modelconf_obj=oconf\<rparr>"


primrec rel_disjoint::"SubjAttrConf\<Rightarrow>ObjAttrConf\<Rightarrow>bool" where
"rel_disjoint sconf noobjattrconf=True"
|"rel_disjoint sconf (objattr_conf oconf oattr)=
(if get_subjattr sconf (objattr_objid oattr)=nosubjattr\<and>
 rel_disjoint sconf oconf
then
True
else False)"

definition valid_modelconf::"ModelConf\<Rightarrow>bool" where
"valid_modelconf mconf\<equiv>(rel_disjoint (modelconf_subj mconf) (modelconf_obj mconf)\<and>
                         valid_subjattrconf(modelconf_subj mconf)\<and>
                         valid_objattrconf(modelconf_obj mconf))"

interpretation ModelConf : ModelConf nousrid valid_usrid noresrcid valid_resrcid noinfoid 
                           valid_infoid InSgx OutSgx is_insgx Device Normal is_normal Data
                           Func is_data noresrcattr resrc_attr presrc_id
                           info_id trust_level presrc_type info_type nousrattr usr_attr usrattr_id
                           nousrattrconf usrattr_conf is_usrattrconf find_usrid
                           delete_usrattr get_usrattr valid_usrattrconf nosubjattr
                           subj_attr subj_callerattr subj_participants subj_resrcattr 
                           nosubjattrconf subjattr_conf is_subjattrconf subjattr_subjid find_subjid
                           delete_subjattr get_subjattr subjattrconf_uniq
                           valid_subjattrconf noobjattr obj_attr 
                           obj_owners obj_resrcattr noobjattrconf objattr_conf
                           is_objattrconf objattr_objid find_objid delete_objattr
                           get_objattr valid_objattrconf rel_disjoint nomodelconf
                           model_conf modelconf_subj modelconf_obj valid_modelconf
proof
  fix conf1
  show "rel_disjoint conf1 noobjattrconf" by auto
next
  fix conf1
  fix attr2
  fix conf2
  show "get_subjattr conf1 (objattr_objid attr2) = nosubjattr \<and>
       rel_disjoint conf1 conf2 \<Longrightarrow>
       rel_disjoint conf1 (objattr_conf conf2 attr2)" by auto
next
  fix conf1
  fix attr2
  fix conf2
  show "get_subjattr conf1 (objattr_objid attr2) \<noteq> nosubjattr \<and>
       rel_disjoint conf1 conf2 \<Longrightarrow>
       \<not> rel_disjoint conf1 (objattr_conf conf2 attr2)" by auto
next
  fix conf1
  fix attr2
  fix conf2
  show "get_subjattr conf1 (objattr_objid attr2) = nosubjattr \<and>
       \<not> rel_disjoint conf1 conf2 \<Longrightarrow>
       \<not> rel_disjoint conf1 (objattr_conf conf2 attr2)" by auto
next
  show "modelconf_subj nomodelconf = nosubjattrconf" by (auto simp:nomodelconf_def)
next
  show "modelconf_obj nomodelconf = noobjattrconf" by (auto simp:nomodelconf_def)
next
  show "\<not> valid_modelconf nomodelconf" by (auto simp add:nomodelconf_def valid_modelconf_def)
next
  fix sconf
  fix oconf
  show "modelconf_subj (model_conf sconf oconf) = sconf" by (auto simp: model_conf_def)
next
  fix sconf
  fix oconf
  show "modelconf_obj (model_conf sconf oconf) = oconf" by (auto simp: model_conf_def)
next
  fix x
  show "\<exists>sconf oconf.
       x = model_conf sconf oconf \<or> x = nomodelconf"
  proof (rule exI)+
    show "x=model_conf (modelconf_subj x) (modelconf_obj x) \<or> x = nomodelconf"
      by (auto simp:model_conf_def)
  qed
next
  fix sconf
  fix oconf
  show "rel_disjoint sconf oconf \<and>
       valid_subjattrconf sconf \<and> valid_objattrconf oconf \<Longrightarrow>
       valid_modelconf (model_conf sconf oconf)" 
    by (auto simp add: model_conf_def valid_modelconf_def)
next
  fix sconf
  fix oconf
  show "\<not> rel_disjoint sconf oconf \<and>
       valid_subjattrconf sconf \<and> valid_objattrconf oconf \<Longrightarrow>
       \<not> valid_modelconf (model_conf sconf oconf)"
    by (auto simp add: model_conf_def valid_modelconf_def)
next
  fix sconf
  fix oconf
  show "rel_disjoint sconf oconf \<and>
       \<not> valid_subjattrconf sconf \<and> valid_objattrconf oconf \<Longrightarrow>
       \<not>valid_modelconf (model_conf sconf oconf)"
    by (auto simp add: model_conf_def valid_modelconf_def)
next
  fix sconf
  fix oconf
  show "rel_disjoint sconf oconf \<and>
       valid_subjattrconf sconf \<and> \<not> valid_objattrconf oconf \<Longrightarrow>
       \<not> valid_modelconf (model_conf sconf oconf)"
    by (auto simp add: model_conf_def valid_modelconf_def)
qed
  




end