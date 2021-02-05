theory TeaclaveRequirements
  imports Main TeaclaveAccessControl
begin

locale TeaclaveRequirements=TeaclaveAccessControl
begin

theorem TEACLAVESRS1:"request_user_access_data mconf uid oid=
                     (find_usrid(objattr_owners(read_objattr mconf oid)) (usr_attr uid)\<and>
                     resrcattr_infotype(objattr_resrcattr(read_objattr mconf oid))=data)"
proof (simp add:read_objattr_def request_user_access_data_def read_subjattr_def)
  show "user_access_data (usr_attr uid) (get_objattr (modelconf_obj mconf) oid) =
       (find_usrid(objattr_owners (get_objattr (modelconf_obj mconf) oid)) (usr_attr uid) \<and>
        resrcattr_infotype(objattr_resrcattr (get_objattr (modelconf_obj mconf) oid))=data)"
  proof
    assume "user_access_data (usr_attr uid) (get_objattr (modelconf_obj mconf) oid)"
    from this show "find_usrid(objattr_owners((get_objattr (modelconf_obj mconf) oid))) (usr_attr uid)\<and>
                   resrcattr_infotype(objattr_resrcattr (get_objattr (modelconf_obj mconf) oid))=data"
      proof (rule contrapos_pp)
        assume "\<not> (find_usrid
                    (objattr_owners(get_objattr (modelconf_obj mconf) oid))
                    (usr_attr uid) \<and>
              resrcattr_infotype(objattr_resrcattr(get_objattr (modelconf_obj mconf) oid))=data)"
        from this show "\<not> user_access_data (usr_attr uid) (get_objattr (modelconf_obj mconf) oid)"
          by (rule FDPACF1HLR13)
      qed
    next
    assume "find_usrid(objattr_owners (get_objattr (modelconf_obj mconf) oid)) (usr_attr uid) \<and>
           resrcattr_infotype(objattr_resrcattr (get_objattr (modelconf_obj mconf) oid))=data"
    from this show "user_access_data (usr_attr uid) (get_objattr (modelconf_obj mconf) oid)"
      by (rule FDPACF1HLR10)
  qed
qed

theorem TEACLAVESRS2:"request_task_access_data mconf sid oid\<Longrightarrow>
                     ((rel_subset(subjattr_participants(read_subjattr mconf sid)) 
                                 (objattr_owners(read_objattr mconf oid)))\<and>
                     resrcattr_infotype(objattr_resrcattr(read_objattr mconf oid))=data)"
proof (simp add:read_objattr_def request_task_access_data_def read_subjattr_def)
  assume 0:"task_access_data (get_subjattr (modelconf_subj mconf) sid)
           (get_objattr (modelconf_obj mconf) oid)"
  from this have 1:"rel_subset(subjattr_participants(get_subjattr (modelconf_subj mconf) sid)) 
                   (objattr_owners(get_objattr (modelconf_obj mconf) oid))\<and>
                   resrcattr_infotype(objattr_resrcattr(get_objattr (modelconf_obj mconf) oid))=data"
  proof (rule contrapos_pp)
    assume "\<not> (rel_subset
           (subjattr_participants(get_subjattr (modelconf_subj mconf) sid))
           (objattr_owners (get_objattr (modelconf_obj mconf) oid)) \<and>
           resrcattr_infotype (objattr_resrcattr(get_objattr (modelconf_obj mconf) oid)) = data)"
    from this show "\<not> task_access_data (get_subjattr (modelconf_subj mconf) sid)
                   (get_objattr (modelconf_obj mconf) oid)" by (rule FDPACF1HLR14)
  qed
  from this show "rel_subset(subjattr_participants(get_subjattr (modelconf_subj mconf) sid))
                 (objattr_owners (get_objattr (modelconf_obj mconf) oid)) \<and>
                 resrcattr_infotype(objattr_resrcattr (get_objattr (modelconf_obj mconf) oid))=data"
    by auto
qed











end



print_locale! TeaclaveRequirements





end