theory I_FDP_ACC
  imports Main "../FDP_ACC" I_ModelConf
begin

interpretation FdpAcc1 : FdpAcc1 nousrid valid_usrid noresrcid valid_resrcid noinfoid valid_infoid
                                 InSgx OutSgx is_insgx Device Normal is_normal Data
                                 Func is_data noresrcattr resrc_attr presrc_id
                                 info_id trust_level presrc_type info_type nousrattr usr_attr 
                                 usrattr_id nousrattrconf usrattr_conf is_usrattrconf find_usrid
                                 delete_usrattr get_usrattr valid_usrattrconf nosubjattr
                                 subj_attr subj_callerattr subj_participants 
                                 subj_resrcattr nosubjattrconf subjattr_conf is_subjattrconf 
                                 subjattr_subjid find_subjid delete_subjattr get_subjattr 
                                 subjattrconf_uniq valid_subjattrconf noobjattr obj_attr 
                                 obj_owners obj_resrcattr noobjattrconf objattr_conf
                                 is_objattrconf objattr_objid find_objid delete_objattr
                                 get_objattr valid_objattrconf rel_disjoint nomodelconf
                                 model_conf modelconf_subj modelconf_obj valid_modelconf
proof

qed




end